use std::collections::HashMap;

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::models::{NewTask, TagRow, TaskChangeset, TaskRow};
use north_db::schema::{projects, tags, task_tags, tasks, users};
use north_db::DbPool;
use north_domain::{CreateTask, TagInfo, Task, TaskFilter, UpdateTask, UserSettings};

use crate::{ServiceError, ServiceResult};

pub struct TaskService;

impl TaskService {
    pub async fn list(
        pool: &DbPool,
        user_id: i64,
        filter: &TaskFilter,
    ) -> ServiceResult<Vec<Task>> {
        let mut conn = pool.get().await?;

        let mut query = tasks::table
            .left_join(projects::table.on(projects::id.nullable().eq(tasks::project_id)))
            .filter(tasks::user_id.eq(user_id))
            .into_boxed();

        if let Some(project_id) = filter.project {
            query = query.filter(tasks::project_id.eq(project_id));
        }
        if let Some(parent_id) = filter.parent {
            query = query.filter(tasks::parent_id.eq(parent_id));
        }
        if filter.inbox == Some(true) {
            query = query.filter(tasks::project_id.is_null());
            query = query.filter(tasks::parent_id.is_null());
        }
        if filter.completed == Some(true) {
            query = query.filter(tasks::completed_at.is_not_null());
        } else if filter.completed == Some(false) {
            query = query.filter(tasks::completed_at.is_null());
        }
        if let Some(ref q) = filter.q {
            let pattern = format!("%{q}%");
            query = query.filter(
                tasks::title
                    .ilike(pattern.clone())
                    .or(tasks::body.ilike(pattern)),
            );
        }
        if filter.review_due == Some(true) {
            let settings_val: serde_json::Value = users::table
                .filter(users::id.eq(user_id))
                .select(users::settings)
                .first(&mut conn)
                .await?;
            let settings: UserSettings = serde_json::from_value(settings_val).unwrap_or_default();
            let cutoff = Utc::now().date_naive()
                - chrono::Duration::days(settings.review_interval_days as i64);
            query = query.filter(
                tasks::reviewed_at
                    .is_null()
                    .or(tasks::reviewed_at.le(cutoff)),
            );
            query = query.filter(tasks::completed_at.is_null());
        }

        // Tag filtering via subquery
        if let Some(ref tag_names) = filter.tag {
            if !tag_names.is_empty() {
                let tag_task_ids: Vec<i64> = task_tags::table
                    .inner_join(tags::table.on(tags::id.eq(task_tags::tag_id)))
                    .filter(tags::name.eq_any(tag_names))
                    .select(task_tags::task_id)
                    .load(&mut conn)
                    .await?;
                query = query.filter(tasks::id.eq_any(tag_task_ids));
            }
        }

        // Sort
        let rows = match filter.sort.as_deref() {
            Some("due_date") => {
                query
                    .order((tasks::due_date.asc().nulls_last(), tasks::sort_key.asc()))
                    .select(TaskRow::as_select())
                    .load(&mut conn)
                    .await?
            }
            Some("created_at") => {
                query
                    .order(tasks::created_at.desc())
                    .select(TaskRow::as_select())
                    .load(&mut conn)
                    .await?
            }
            Some("title") => {
                query
                    .order(tasks::title.asc())
                    .select(TaskRow::as_select())
                    .load(&mut conn)
                    .await?
            }
            _ => {
                query
                    .order((tasks::sort_key.asc(), tasks::created_at.desc()))
                    .select(TaskRow::as_select())
                    .load(&mut conn)
                    .await?
            }
        };

        // Apply limit/offset
        let rows = if let Some(offset) = filter.offset {
            rows.into_iter().skip(offset as usize).collect()
        } else {
            rows
        };
        let rows = if let Some(limit) = filter.limit {
            rows.into_iter().take(limit as usize).collect()
        } else {
            rows
        };

        let mut results = Self::load_with_meta(pool, rows).await?;
        Self::compute_actionable_batch(pool, &mut results).await?;

        if filter.actionable == Some(true) {
            results.retain(|t| t.actionable);
        }

        Ok(results)
    }

    pub async fn get_by_id(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<Task> {
        let mut conn = pool.get().await?;
        let row = tasks::table
            .filter(tasks::id.eq(id))
            .filter(tasks::user_id.eq(user_id))
            .select(TaskRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Task not found".into()))?;
        let mut results = Self::load_with_meta(pool, vec![row]).await?;

        if let Some(item) = results.first_mut() {
            item.actionable = Self::compute_actionable_single(pool, item).await?;
        }

        results
            .into_iter()
            .next()
            .ok_or_else(|| ServiceError::NotFound("Task not found".into()))
    }

    pub async fn create(pool: &DbPool, user_id: i64, input: &CreateTask) -> ServiceResult<Task> {
        let mut conn = pool.get().await?;

        let last_key: Option<String> = if input.parent_id.is_some() {
            tasks::table
                .filter(tasks::parent_id.eq(input.parent_id))
                .filter(tasks::user_id.eq(user_id))
                .order(tasks::sort_key.desc())
                .select(tasks::sort_key)
                .first(&mut conn)
                .await
                .optional()?
        } else if input.project_id.is_some() {
            tasks::table
                .filter(tasks::project_id.eq(input.project_id))
                .filter(tasks::parent_id.is_null())
                .filter(tasks::user_id.eq(user_id))
                .order(tasks::sort_key.desc())
                .select(tasks::sort_key)
                .first(&mut conn)
                .await
                .optional()?
        } else {
            tasks::table
                .filter(tasks::project_id.is_null())
                .filter(tasks::parent_id.is_null())
                .filter(tasks::user_id.eq(user_id))
                .order(tasks::sort_key.desc())
                .select(tasks::sort_key)
                .first(&mut conn)
                .await
                .optional()?
        };
        let sort_key = north_domain::sort_key_after(last_key.as_deref());

        let row = diesel::insert_into(tasks::table)
            .values(&NewTask {
                user_id,
                title: &input.title,
                body: input.body.as_deref(),
                project_id: input.project_id,
                parent_id: input.parent_id,
                sort_key: &sort_key,
                start_at: input.start_at,
                due_date: input.due_date,
            })
            .returning(TaskRow::as_returning())
            .get_result(&mut conn)
            .await?;

        let mut task = Task::from(row);

        // Enrich with project_title
        if let Some(pid) = task.project_id {
            task.project_title = projects::table
                .filter(projects::id.eq(pid))
                .select(projects::title)
                .first::<String>(&mut conn)
                .await
                .ok();
        }

        // Compute actionable for new task
        task.actionable = Self::compute_actionable_single(pool, &task).await?;

        Ok(task)
    }

    pub async fn update(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        input: &UpdateTask,
    ) -> ServiceResult<Task> {
        let mut conn = pool.get().await?;

        let existing = tasks::table
            .filter(tasks::id.eq(id))
            .filter(tasks::user_id.eq(user_id))
            .select(TaskRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Task not found".into()))?;

        let mut changeset = TaskChangeset::default();

        if let Some(ref title) = input.title {
            changeset.title = Some(title.as_str());
        }
        if let Some(ref body) = input.body {
            changeset.body = Some(body.as_deref());
        }
        if let Some(ref project_id) = input.project_id {
            changeset.project_id = Some(*project_id);
        }
        if let Some(ref parent_id) = input.parent_id {
            changeset.parent_id = Some(*parent_id);
            if let Some(pid) = parent_id {
                let parent_project: Option<i64> = tasks::table
                    .filter(tasks::id.eq(*pid))
                    .select(tasks::project_id)
                    .first(&mut conn)
                    .await
                    .optional()?
                    .flatten();
                changeset.project_id = Some(parent_project);
            }
        }
        if let Some(ref sort_key) = input.sort_key {
            changeset.sort_key = Some(sort_key.as_str());
        }

        // When project or parent changes and no explicit sort_key was
        // provided, reset sort_key so the task lands at the bottom.
        let new_sort_key;
        let resolved_project = changeset.project_id.unwrap_or(existing.project_id);
        let resolved_parent = changeset.parent_id.unwrap_or(existing.parent_id);
        let project_changed =
            resolved_project != existing.project_id || resolved_parent != existing.parent_id;
        if project_changed && input.sort_key.is_none() {
            let last_key: Option<String> = if let Some(pid) = resolved_parent {
                tasks::table
                    .filter(tasks::parent_id.eq(pid))
                    .filter(tasks::user_id.eq(user_id))
                    .filter(tasks::id.ne(id))
                    .order(tasks::sort_key.desc())
                    .select(tasks::sort_key)
                    .first(&mut conn)
                    .await
                    .optional()?
            } else if let Some(proj) = resolved_project {
                tasks::table
                    .filter(tasks::project_id.eq(proj))
                    .filter(tasks::parent_id.is_null())
                    .filter(tasks::user_id.eq(user_id))
                    .filter(tasks::id.ne(id))
                    .order(tasks::sort_key.desc())
                    .select(tasks::sort_key)
                    .first(&mut conn)
                    .await
                    .optional()?
            } else {
                tasks::table
                    .filter(tasks::project_id.is_null())
                    .filter(tasks::parent_id.is_null())
                    .filter(tasks::user_id.eq(user_id))
                    .filter(tasks::id.ne(id))
                    .order(tasks::sort_key.desc())
                    .select(tasks::sort_key)
                    .first(&mut conn)
                    .await
                    .optional()?
            };
            new_sort_key = north_domain::sort_key_after(last_key.as_deref());
            changeset.sort_key = Some(&new_sort_key);
        }
        if let Some(sequential_limit) = input.sequential_limit {
            changeset.sequential_limit = Some(sequential_limit);
        }
        if let Some(ref start_at) = input.start_at {
            changeset.start_at = Some(*start_at);
        }
        if let Some(ref due_date) = input.due_date {
            changeset.due_date = Some(*due_date);
        }
        if let Some(ref reviewed_at) = input.reviewed_at {
            changeset.reviewed_at = Some(*reviewed_at);
        }
        if let Some(ref completed_at) = input.completed_at {
            changeset.completed_at = Some(*completed_at);
        }

        // When completing and no explicit sort_key, reset to empty.
        // When uncompleting and no explicit sort_key, place at end of list.
        let uncomplete_sort_key: String;
        if input.sort_key.is_none() && changeset.sort_key.is_none() {
            if let Some(Some(_)) = input.completed_at {
                if existing.completed_at.is_none() {
                    changeset.sort_key = Some("");
                }
            } else if let Some(None) = input.completed_at {
                if existing.completed_at.is_some() {
                    let last_key: Option<String> = if let Some(pid) = resolved_parent {
                        tasks::table
                            .filter(tasks::parent_id.eq(pid))
                            .filter(tasks::user_id.eq(user_id))
                            .filter(tasks::completed_at.is_null())
                            .filter(tasks::id.ne(id))
                            .order(tasks::sort_key.desc())
                            .select(tasks::sort_key)
                            .first(&mut conn)
                            .await
                            .optional()?
                    } else if let Some(proj) = resolved_project {
                        tasks::table
                            .filter(tasks::project_id.eq(proj))
                            .filter(tasks::parent_id.is_null())
                            .filter(tasks::user_id.eq(user_id))
                            .filter(tasks::completed_at.is_null())
                            .filter(tasks::id.ne(id))
                            .order(tasks::sort_key.desc())
                            .select(tasks::sort_key)
                            .first(&mut conn)
                            .await
                            .optional()?
                    } else {
                        tasks::table
                            .filter(tasks::project_id.is_null())
                            .filter(tasks::parent_id.is_null())
                            .filter(tasks::user_id.eq(user_id))
                            .filter(tasks::completed_at.is_null())
                            .filter(tasks::id.ne(id))
                            .order(tasks::sort_key.desc())
                            .select(tasks::sort_key)
                            .first(&mut conn)
                            .await
                            .optional()?
                    };
                    uncomplete_sort_key = north_domain::sort_key_after(last_key.as_deref());
                    changeset.sort_key = Some(&uncomplete_sort_key);
                }
            }
        }

        let row = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(&changeset)
        .returning(TaskRow::as_returning())
        .get_result(&mut conn)
        .await?;

        // If completing (was null, now set), cascade to descendants
        if let Some(Some(_)) = input.completed_at {
            if existing.completed_at.is_none() {
                let now = Utc::now();
                let mut parent_ids = vec![id];
                for _ in 0..5 {
                    if parent_ids.is_empty() {
                        break;
                    }
                    let child_ids: Vec<i64> = tasks::table
                        .filter(tasks::parent_id.eq_any(&parent_ids))
                        .filter(tasks::completed_at.is_null())
                        .select(tasks::id)
                        .load(&mut conn)
                        .await?;
                    if child_ids.is_empty() {
                        break;
                    }
                    diesel::update(tasks::table.filter(tasks::id.eq_any(&child_ids)))
                        .set(tasks::completed_at.eq(Some(now)))
                        .execute(&mut conn)
                        .await?;
                    parent_ids = child_ids;
                }
            }
        }

        Ok(Task::from(row))
    }

    pub async fn delete(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::delete(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn review(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<Task> {
        let today = Utc::now().date_naive();
        let mut conn = pool.get().await?;
        let row = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::reviewed_at.eq(Some(today)))
        .returning(TaskRow::as_returning())
        .get_result(&mut conn)
        .await
        .optional()?
        .ok_or_else(|| ServiceError::NotFound("Task not found".into()))?;
        Ok(Task::from(row))
    }

    // ── Internal helpers ───────────────────────────────────────────

    async fn load_with_meta(pool: &DbPool, task_rows: Vec<TaskRow>) -> ServiceResult<Vec<Task>> {
        if task_rows.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = pool.get().await?;
        let task_ids: Vec<i64> = task_rows.iter().map(|t| t.id).collect();

        // Batch load project titles
        let project_ids: Vec<i64> = task_rows
            .iter()
            .filter_map(|t| t.project_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let proj_map: HashMap<i64, String> = if !project_ids.is_empty() {
            projects::table
                .filter(projects::id.eq_any(&project_ids))
                .select((projects::id, projects::title))
                .load::<(i64, String)>(&mut conn)
                .await?
                .into_iter()
                .collect()
        } else {
            HashMap::new()
        };

        // Batch load tags via join table
        let tag_rows: Vec<(i64, TagRow)> = task_tags::table
            .inner_join(tags::table.on(tags::id.eq(task_tags::tag_id)))
            .filter(task_tags::task_id.eq_any(&task_ids))
            .select((task_tags::task_id, TagRow::as_select()))
            .load(&mut conn)
            .await?;

        let mut tags_map: HashMap<i64, Vec<TagInfo>> = HashMap::new();
        for (task_id, tag) in tag_rows {
            tags_map
                .entry(task_id)
                .or_default()
                .push(TagInfo::from(&tag));
        }

        // Batch load subtask counts
        let counts: Vec<(Option<i64>, i64)> = tasks::table
            .filter(tasks::parent_id.eq_any(&task_ids))
            .group_by(tasks::parent_id)
            .select((tasks::parent_id, diesel::dsl::count_star()))
            .load(&mut conn)
            .await?;
        let count_map: HashMap<i64, i64> = counts
            .into_iter()
            .filter_map(|(pid, cnt)| pid.map(|id| (id, cnt)))
            .collect();

        // Batch load completed subtask counts
        let completed_counts: Vec<(Option<i64>, i64)> = tasks::table
            .filter(tasks::parent_id.eq_any(&task_ids))
            .filter(tasks::completed_at.is_not_null())
            .group_by(tasks::parent_id)
            .select((tasks::parent_id, diesel::dsl::count_star()))
            .load(&mut conn)
            .await?;
        let completed_count_map: HashMap<i64, i64> = completed_counts
            .into_iter()
            .filter_map(|(pid, cnt)| pid.map(|id| (id, cnt)))
            .collect();

        Ok(task_rows
            .into_iter()
            .map(|row| {
                let id = row.id;
                let project_title = row.project_id.and_then(|pid| proj_map.get(&pid).cloned());
                let tags = tags_map.remove(&id).unwrap_or_default();
                let subtask_count = count_map.get(&id).copied().unwrap_or(0);
                let completed_subtask_count = completed_count_map.get(&id).copied().unwrap_or(0);
                let actionable = row.completed_at.is_none();
                let mut task = Task::from(row);
                task.project_title = project_title;
                task.tags = tags;
                task.subtask_count = subtask_count;
                task.completed_subtask_count = completed_subtask_count;
                task.actionable = actionable;
                task
            })
            .collect())
    }

    async fn compute_actionable_single(pool: &DbPool, task: &Task) -> ServiceResult<bool> {
        if task.completed_at.is_some() {
            return Ok(false);
        }
        if let Some(start) = task.start_at {
            if start.date_naive() > Utc::now().date_naive() {
                return Ok(false);
            }
        }
        if task.parent_id.is_none() {
            return Ok(true);
        }

        let mut conn = pool.get().await?;
        let parent_id = task.parent_id.unwrap();

        let parent_limit: i16 = tasks::table
            .filter(tasks::id.eq(parent_id))
            .select(tasks::sequential_limit)
            .first(&mut conn)
            .await
            .unwrap_or(1);

        let siblings_before: i64 = tasks::table
            .filter(tasks::parent_id.eq(parent_id))
            .filter(tasks::completed_at.is_null())
            .filter(tasks::sort_key.lt(&task.sort_key))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(siblings_before < parent_limit as i64)
    }

    async fn compute_actionable_batch(pool: &DbPool, results: &mut [Task]) -> ServiceResult<()> {
        let today = Utc::now().date_naive();

        let parent_ids: Vec<i64> = results
            .iter()
            .filter_map(|r| r.parent_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let _parent_limits: HashMap<i64, i16> = if !parent_ids.is_empty() {
            let mut conn = pool.get().await?;
            tasks::table
                .filter(tasks::id.eq_any(&parent_ids))
                .select((tasks::id, tasks::sequential_limit))
                .load::<(i64, i16)>(&mut conn)
                .await?
                .into_iter()
                .collect()
        } else {
            HashMap::new()
        };

        for item in results.iter_mut() {
            if item.completed_at.is_some() {
                item.actionable = false;
                continue;
            }
            if let Some(start) = item.start_at {
                if start.date_naive() > today {
                    item.actionable = false;
                    continue;
                }
            }
            if item.parent_id.is_none() {
                item.actionable = true;
                continue;
            }
            item.actionable = true;
        }

        Ok(())
    }
}
