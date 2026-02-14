use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::models::{NewTask, TagRow, TaskChangeset, TaskRow};
use north_db::schema::{projects, tags, task_tags, tasks, users};
use north_db::DbPool;
use north_domain::{CreateTask, TagInfo, Task, TaskFilter, TaskWithMeta, UpdateTask, UserSettings};

use crate::tag_service::TagService;
use crate::{ServiceError, ServiceResult};

pub struct TaskService;

impl TaskService {
    // ── Query methods ──────────────────────────────────────────────

    pub async fn get_inbox(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<TaskWithMeta>> {
        let mut conn = pool.get().await?;
        let rows = tasks::table
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::project_id.is_null())
            .filter(tasks::parent_id.is_null())
            .filter(tasks::completed_at.is_null())
            .order(tasks::sort_key.asc())
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;
        Self::enrich(pool, rows).await
    }

    pub async fn get_today(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<TaskWithMeta>> {
        let mut conn = pool.get().await?;
        let today = Utc::now().date_naive();

        // Load tasks with start_at <= today, not completed, not in archived projects
        let rows = tasks::table
            .left_join(projects::table.on(projects::id.nullable().eq(tasks::project_id)))
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::start_at.is_not_null())
            .filter(tasks::completed_at.is_null())
            .filter(
                tasks::project_id
                    .is_null()
                    .or(projects::status.eq(north_db::sql_types::ProjectStatusMapping::Active)),
            )
            .order(tasks::start_at.asc())
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;

        // Post-filter: start_at::date <= today (Diesel can't do ::date cast easily)
        let rows: Vec<TaskRow> = rows
            .into_iter()
            .filter(|r| {
                r.start_at
                    .map(|dt| dt.date_naive() <= today)
                    .unwrap_or(false)
            })
            .collect();

        Self::enrich(pool, rows).await
    }

    pub async fn get_all(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<TaskWithMeta>> {
        let mut conn = pool.get().await?;
        let rows = tasks::table
            .left_join(projects::table.on(projects::id.nullable().eq(tasks::project_id)))
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::parent_id.is_null())
            .filter(
                tasks::project_id
                    .is_null()
                    .or(projects::status.eq(north_db::sql_types::ProjectStatusMapping::Active)),
            )
            .order((tasks::sort_key.asc(), tasks::created_at.desc()))
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;

        let mut results = Self::enrich(pool, rows).await?;

        // Sort: uncompleted first, then by position; completed last by completed_at desc
        results.sort_by(|a, b| {
            let a_done = a.task.completed_at.is_some();
            let b_done = b.task.completed_at.is_some();
            match (a_done, b_done) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                (false, false) => a.task.sort_key.cmp(&b.task.sort_key),
                (true, true) => b.task.completed_at.cmp(&a.task.completed_at),
            }
        });

        // Set actionable flag
        for item in &mut results {
            item.actionable = item.task.completed_at.is_none();
        }

        Ok(results)
    }

    pub async fn get_for_project(
        pool: &DbPool,
        user_id: i64,
        project_id: i64,
    ) -> ServiceResult<Vec<TaskWithMeta>> {
        let mut conn = pool.get().await?;
        let rows = tasks::table
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::project_id.eq(project_id))
            .filter(tasks::parent_id.is_null())
            .filter(tasks::completed_at.is_null())
            .order(tasks::sort_key.asc())
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;
        Self::enrich(pool, rows).await
    }

    pub async fn get_review_due(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<TaskWithMeta>> {
        let mut conn = pool.get().await?;

        // Get user's review interval
        let settings_val: serde_json::Value = users::table
            .filter(users::id.eq(user_id))
            .select(users::settings)
            .first(&mut conn)
            .await?;
        let settings: UserSettings = serde_json::from_value(settings_val).unwrap_or_default();
        let interval_days = settings.review_interval_days as i64;
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(interval_days);

        let rows = tasks::table
            .left_join(projects::table.on(projects::id.nullable().eq(tasks::project_id)))
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::parent_id.is_null())
            .filter(tasks::completed_at.is_null())
            .filter(
                tasks::project_id
                    .is_null()
                    .or(projects::status.eq(north_db::sql_types::ProjectStatusMapping::Active)),
            )
            .filter(
                tasks::reviewed_at
                    .is_null()
                    .or(tasks::reviewed_at.le(cutoff)),
            )
            .order((
                tasks::reviewed_at.asc().nulls_first(),
                tasks::sort_key.asc(),
            ))
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;

        Self::enrich(pool, rows).await
    }

    pub async fn get_recently_reviewed(
        pool: &DbPool,
        user_id: i64,
    ) -> ServiceResult<Vec<TaskWithMeta>> {
        let mut conn = pool.get().await?;
        let rows = tasks::table
            .left_join(projects::table.on(projects::id.nullable().eq(tasks::project_id)))
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::parent_id.is_null())
            .filter(tasks::completed_at.is_null())
            .filter(tasks::reviewed_at.is_not_null())
            .filter(
                tasks::project_id
                    .is_null()
                    .or(projects::status.eq(north_db::sql_types::ProjectStatusMapping::Active)),
            )
            .order(tasks::reviewed_at.desc())
            .limit(50)
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;
        Self::enrich(pool, rows).await
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
        Ok(Task::from(row))
    }

    pub async fn get_by_id_with_meta(
        pool: &DbPool,
        user_id: i64,
        id: i64,
    ) -> ServiceResult<TaskWithMeta> {
        let mut conn = pool.get().await?;
        let row = tasks::table
            .filter(tasks::id.eq(id))
            .filter(tasks::user_id.eq(user_id))
            .select(TaskRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Task not found".into()))?;
        let mut results = Self::enrich(pool, vec![row]).await?;

        // Compute actionable for single task
        if let Some(item) = results.first_mut() {
            item.actionable = Self::compute_actionable_single(pool, &item.task).await?;
        }

        results
            .into_iter()
            .next()
            .ok_or_else(|| ServiceError::NotFound("Task not found".into()))
    }

    pub async fn list(
        pool: &DbPool,
        user_id: i64,
        filter: &TaskFilter,
    ) -> ServiceResult<Vec<TaskWithMeta>> {
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
            // Get review interval
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

        // Apply limit/offset in Rust (Diesel boxed queries with dynamic limit are tricky)
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

        let mut results = Self::enrich(pool, rows).await?;

        // Compute actionable
        Self::compute_actionable_batch(pool, &mut results).await?;

        // Post-filter actionable if requested
        if filter.actionable == Some(true) {
            results.retain(|t| t.actionable);
        }

        Ok(results)
    }

    // ── Mutation methods ───────────────────────────────────────────

    pub async fn create_task(
        pool: &DbPool,
        user_id: i64,
        title: String,
        body: Option<String>,
    ) -> ServiceResult<Task> {
        let mut conn = pool.get().await?;

        let title_parsed = north_domain::parse_tokens(&title);
        let body_parsed = body
            .as_deref()
            .filter(|b| !b.trim().is_empty())
            .map(north_domain::parse_tokens);

        let clean_title = title_parsed.cleaned;
        let clean_body = body_parsed
            .as_ref()
            .map(|p| p.cleaned.clone())
            .filter(|b| !b.trim().is_empty());

        let mut all_tags: Vec<String> = title_parsed.tags;
        if let Some(ref bp) = body_parsed {
            for t in &bp.tags {
                let lower = t.to_lowercase();
                if !all_tags.contains(&lower) {
                    all_tags.push(lower);
                }
            }
        }

        let project_name = title_parsed
            .project
            .or_else(|| body_parsed.and_then(|p| p.project));

        // Resolve @project by name
        let mut project_id: Option<i64> = None;
        if let Some(ref name) = project_name {
            project_id = crate::ProjectService::find_by_title(pool, user_id, name).await?;
        }

        // Get sort key after last sibling
        let last_key: Option<String> = tasks::table
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::project_id.is_null())
            .filter(tasks::parent_id.is_null())
            .order(tasks::sort_key.desc())
            .select(tasks::sort_key)
            .first(&mut conn)
            .await
            .optional()?;
        let sort_key = north_domain::sort_key_after(last_key.as_deref());

        let task_row = diesel::insert_into(tasks::table)
            .values(&NewTask {
                user_id,
                title: &clean_title,
                body: clean_body.as_deref(),
                project_id,
                parent_id: None,
                sort_key: &sort_key,
                start_at: None,
                due_date: None,
            })
            .returning(TaskRow::as_returning())
            .get_result(&mut conn)
            .await?;

        if !all_tags.is_empty() {
            TagService::add_task_tags(&mut conn, user_id, task_row.id, &all_tags).await?;
        }

        Ok(Task::from(task_row))
    }

    pub async fn create_task_full(
        pool: &DbPool,
        user_id: i64,
        input: &CreateTask,
    ) -> ServiceResult<Task> {
        let mut conn = pool.get().await?;

        // Determine sort key context
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

        Ok(Task::from(row))
    }

    pub async fn update_task(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        title: String,
        body: Option<String>,
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;

        let title_parsed = north_domain::parse_tokens(&title);
        let body_parsed = body
            .as_deref()
            .filter(|b| !b.trim().is_empty())
            .map(north_domain::parse_tokens);

        let clean_title = title_parsed.cleaned;
        let clean_body = body_parsed
            .as_ref()
            .map(|p| p.cleaned.clone())
            .filter(|b| !b.trim().is_empty());

        let mut new_tags: Vec<String> = title_parsed.tags;
        if let Some(ref bp) = body_parsed {
            for t in &bp.tags {
                let lower = t.to_lowercase();
                if !new_tags.contains(&lower) {
                    new_tags.push(lower);
                }
            }
        }

        let project_name = title_parsed
            .project
            .or_else(|| body_parsed.and_then(|p| p.project));

        // Conditionally set project
        let mut changeset = TaskChangeset {
            title: Some(&clean_title),
            body: Some(clean_body.as_deref()),
            ..Default::default()
        };

        if let Some(ref name) = project_name {
            if let Some(pid) = crate::ProjectService::find_by_title(pool, user_id, name).await? {
                changeset.project_id = Some(Some(pid));
            }
        }

        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(&changeset)
        .execute(&mut conn)
        .await?;

        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }

        if !new_tags.is_empty() {
            TagService::add_task_tags(&mut conn, user_id, id, &new_tags).await?;
        }

        Ok(())
    }

    pub async fn update_task_full(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        input: &UpdateTask,
    ) -> ServiceResult<Task> {
        let mut conn = pool.get().await?;

        // Fetch existing
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
            // When nesting under a new parent, inherit parent's project_id
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

    pub async fn reorder_task(
        pool: &DbPool,
        user_id: i64,
        task_id: i64,
        sort_key: String,
        parent_id: Option<Option<i64>>,
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;

        let existing = tasks::table
            .filter(tasks::id.eq(task_id))
            .filter(tasks::user_id.eq(user_id))
            .select(TaskRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Task not found".into()))?;

        let mut changeset = TaskChangeset {
            sort_key: Some(&sort_key),
            ..Default::default()
        };

        if let Some(new_parent) = parent_id {
            changeset.parent_id = Some(new_parent);

            // When nesting, inherit parent's project_id
            if let Some(pid) = new_parent {
                let parent_project: Option<i64> = tasks::table
                    .filter(tasks::id.eq(pid))
                    .select(tasks::project_id)
                    .first(&mut conn)
                    .await
                    .optional()?
                    .flatten();
                changeset.project_id = Some(parent_project);
            } else if existing.parent_id.is_some() {
                // Unnesting — keep current project_id
            }
        }

        diesel::update(
            tasks::table
                .filter(tasks::id.eq(task_id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(&changeset)
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn complete_task(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let now = Utc::now();
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::completed_at.is_null()),
        )
        .set((
            tasks::completed_at.eq(Some(now)),
            tasks::sort_key.eq(""),
        ))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }

        // Cascade: complete all incomplete descendants (max depth 5)
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
                .set((
                    tasks::completed_at.eq(Some(now)),
                    tasks::sort_key.eq(""),
                ))
                .execute(&mut conn)
                .await?;
            parent_ids = child_ids;
        }

        Ok(())
    }

    pub async fn uncomplete_task(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;

        let existing = tasks::table
            .filter(tasks::id.eq(id))
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::completed_at.is_not_null())
            .select(TaskRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Task not found".into()))?;

        let last_key: Option<String> = if let Some(pid) = existing.parent_id {
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
        } else if let Some(proj) = existing.project_id {
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
        let new_sort_key = north_domain::sort_key_after(last_key.as_deref());

        diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set((
            tasks::completed_at.eq(None::<DateTime<Utc>>),
            tasks::sort_key.eq(&new_sort_key),
        ))
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn delete_task(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
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

    pub async fn set_start_at(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        dt: DateTime<Utc>,
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::start_at.eq(Some(dt)))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn clear_start_at(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::start_at.eq(None::<DateTime<Utc>>))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn set_project(
        pool: &DbPool,
        user_id: i64,
        task_id: i64,
        project_id: i64,
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(task_id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::project_id.eq(Some(project_id)))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn clear_project(pool: &DbPool, user_id: i64, task_id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(task_id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::project_id.eq(None::<i64>))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn get_children(
        pool: &DbPool,
        user_id: i64,
        parent_id: i64,
    ) -> ServiceResult<Vec<TaskWithMeta>> {
        let mut conn = pool.get().await?;
        let rows = tasks::table
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::parent_id.eq(parent_id))
            .order(tasks::sort_key.asc())
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;

        let mut results = Self::enrich(pool, rows).await?;

        // Compute actionable based on parent's sequential_limit
        let parent_limit: i16 = tasks::table
            .filter(tasks::id.eq(parent_id))
            .select(tasks::sequential_limit)
            .first(&mut conn)
            .await
            .unwrap_or(1);

        let mut active_count: i64 = 0;
        for item in &mut results {
            if item.task.completed_at.is_some() {
                item.actionable = false;
            } else {
                item.actionable = active_count < parent_limit as i64;
                active_count += 1;
            }
        }

        Ok(results)
    }

    pub async fn get_ancestors(
        pool: &DbPool,
        user_id: i64,
        id: i64,
    ) -> ServiceResult<Vec<(i64, String, i64)>> {
        let mut conn = pool.get().await?;
        let mut ancestors = Vec::new();
        let mut current_id = id;

        for _ in 0..5 {
            let row = tasks::table
                .filter(tasks::id.eq(current_id))
                .filter(tasks::user_id.eq(user_id))
                .select(TaskRow::as_select())
                .first(&mut conn)
                .await
                .optional()?;

            match row {
                Some(r) => {
                    match r.parent_id {
                        Some(pid) => {
                            // Load parent info
                            let parent = tasks::table
                                .filter(tasks::id.eq(pid))
                                .filter(tasks::user_id.eq(user_id))
                                .select(TaskRow::as_select())
                                .first(&mut conn)
                                .await
                                .optional()?;
                            match parent {
                                Some(p) => {
                                    let child_count: i64 = tasks::table
                                        .filter(tasks::parent_id.eq(p.id))
                                        .count()
                                        .get_result(&mut conn)
                                        .await?;
                                    ancestors.push((p.id, p.title.clone(), child_count));
                                    current_id = p.id;
                                }
                                None => break,
                            }
                        }
                        None => break,
                    }
                }
                None => break,
            }
        }

        ancestors.reverse();
        Ok(ancestors)
    }

    pub async fn set_due_date(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        date: chrono::NaiveDate,
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::due_date.eq(Some(date)))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn clear_due_date(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::due_date.eq(None::<chrono::NaiveDate>))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn set_sequential_limit(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        limit: i16,
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::sequential_limit.eq(limit))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn mark_reviewed(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let today = Utc::now().date_naive();
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            tasks::table
                .filter(tasks::id.eq(id))
                .filter(tasks::user_id.eq(user_id)),
        )
        .set(tasks::reviewed_at.eq(Some(today)))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Task not found".into()));
        }
        Ok(())
    }

    pub async fn review_task_returning(
        pool: &DbPool,
        user_id: i64,
        id: i64,
    ) -> ServiceResult<Task> {
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

    pub async fn mark_all_reviewed(pool: &DbPool, user_id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let today = Utc::now().date_naive();

        // Get review interval
        let settings_val: serde_json::Value = users::table
            .filter(users::id.eq(user_id))
            .select(users::settings)
            .first(&mut conn)
            .await?;
        let settings: UserSettings = serde_json::from_value(settings_val).unwrap_or_default();
        let cutoff =
            Utc::now().date_naive() - chrono::Duration::days(settings.review_interval_days as i64);

        diesel::update(
            tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .filter(tasks::completed_at.is_null())
                .filter(
                    tasks::reviewed_at
                        .is_null()
                        .or(tasks::reviewed_at.le(cutoff)),
                ),
        )
        .set(tasks::reviewed_at.eq(Some(today)))
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn execute_dsl_filter(
        pool: &DbPool,
        user_id: i64,
        query_str: &str,
    ) -> ServiceResult<Vec<TaskWithMeta>> {
        let parsed = north_domain::parse_filter(query_str).map_err(|errs| {
            ServiceError::BadRequest(
                errs.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;

        let mut conn = pool.get().await?;

        // Evaluate expression to get matching task IDs
        let matching_ids: Vec<i64> = if let Some(ref expr) = parsed.expression {
            let ids = crate::filter_translator::eval_expr(pool, user_id, expr).await?;
            ids.into_iter().collect()
        } else {
            // No expression — return all top-level tasks
            tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .select(tasks::id)
                .load(&mut conn)
                .await?
        };

        if matching_ids.is_empty() {
            return Ok(vec![]);
        }

        // Load task rows by IDs
        let rows: Vec<north_db::models::TaskRow> = tasks::table
            .filter(tasks::id.eq_any(&matching_ids))
            .select(north_db::models::TaskRow::as_select())
            .load(&mut conn)
            .await?;

        let mut results = Self::enrich(pool, rows).await?;
        Self::compute_actionable_batch(pool, &mut results).await?;

        // Apply ORDER BY from parsed query
        if let Some(ref order_by) = parsed.order_by {
            use north_domain::SortDirection;
            results.sort_by(|a, b| {
                let cmp = match order_by.field {
                    north_domain::FilterField::Title => a
                        .task
                        .title
                        .to_lowercase()
                        .cmp(&b.task.title.to_lowercase()),
                    north_domain::FilterField::DueDate => a.task.due_date.cmp(&b.task.due_date),
                    north_domain::FilterField::StartAt => a.task.start_at.cmp(&b.task.start_at),
                    north_domain::FilterField::Created => a.task.created_at.cmp(&b.task.created_at),
                    north_domain::FilterField::Updated => a.task.updated_at.cmp(&b.task.updated_at),
                    _ => a.task.sort_key.cmp(&b.task.sort_key),
                };
                match order_by.direction {
                    SortDirection::Asc => cmp,
                    SortDirection::Desc => cmp.reverse(),
                }
            });
        }

        Ok(results)
    }

    pub async fn get_completed(
        pool: &DbPool,
        user_id: i64,
        project_id: Option<i64>,
        inbox_only: bool,
    ) -> ServiceResult<Vec<TaskWithMeta>> {
        let mut conn = pool.get().await?;

        let mut query = tasks::table
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::parent_id.is_null())
            .filter(tasks::completed_at.is_not_null())
            .into_boxed();

        if let Some(pid) = project_id {
            query = query.filter(tasks::project_id.eq(pid));
        } else if inbox_only {
            query = query.filter(tasks::project_id.is_null());
        }

        let rows = query
            .order(tasks::completed_at.desc())
            .limit(50)
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;

        Self::enrich(pool, rows).await
    }

    // ── Internal helpers ───────────────────────────────────────────

    async fn enrich(pool: &DbPool, task_rows: Vec<TaskRow>) -> ServiceResult<Vec<TaskWithMeta>> {
        if task_rows.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = pool.get().await?;
        let task_ids: Vec<i64> = task_rows.iter().map(|t| t.id).collect();

        // 1. Batch load project titles
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

        // 2. Batch load tags via join table
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

        // 3. Batch load subtask counts
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

        // 3b. Batch load completed subtask counts
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

        // 4. Assemble
        Ok(task_rows
            .into_iter()
            .map(|row| {
                let id = row.id;
                TaskWithMeta {
                    project_title: row.project_id.and_then(|pid| proj_map.get(&pid).cloned()),
                    tags: tags_map.remove(&id).unwrap_or_default(),
                    subtask_count: count_map.get(&id).copied().unwrap_or(0),
                    completed_subtask_count: completed_count_map.get(&id).copied().unwrap_or(0),
                    actionable: row.completed_at.is_none(),
                    task: Task::from(row),
                }
            })
            .collect())
    }

    /// Compute actionable for a single task (handles sequential parent logic)
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

        // Sequential logic: count siblings with lower position that are not completed
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

    /// Batch compute actionable for filtered results
    async fn compute_actionable_batch(
        pool: &DbPool,
        results: &mut [TaskWithMeta],
    ) -> ServiceResult<()> {
        let today = Utc::now().date_naive();

        // Collect parent_ids we need to look up
        let parent_ids: Vec<i64> = results
            .iter()
            .filter_map(|r| r.task.parent_id)
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

        // Group tasks by parent for sequential ranking
        let mut parent_groups: HashMap<i64, Vec<usize>> = HashMap::new();
        for (i, item) in results.iter().enumerate() {
            if let Some(pid) = item.task.parent_id {
                parent_groups.entry(pid).or_default().push(i);
            }
        }

        for item in results.iter_mut() {
            let task = &item.task;
            if task.completed_at.is_some() {
                item.actionable = false;
                continue;
            }
            if let Some(start) = task.start_at {
                if start.date_naive() > today {
                    item.actionable = false;
                    continue;
                }
            }
            if task.parent_id.is_none() {
                item.actionable = true;
                continue;
            }

            // For subtasks, use position-based ranking
            // This is a simplified version; full sequential logic would need
            // all siblings loaded. For the filter endpoint, we mark true and
            // let the post-filter handle it.
            item.actionable = true;
        }

        Ok(())
    }
}
