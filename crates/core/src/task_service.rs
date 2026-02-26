use std::collections::HashMap;

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::models::{NewTask, NewTaskTag, TagRow, TaskChangeset, TaskRow};
use north_db::schema::{projects, tags, task_tags, tasks, users};
use north_db::sql_types::RecurrenceTypeMapping;
use north_db::DbPool;
use north_dto::RecurrenceType;
use north_dto::{CreateTask, TagInfo, Task, TaskFilter, UpdateTask, UserSettings};

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

        let results = Self::load_with_meta(pool, rows).await?;

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
        let results = Self::load_with_meta(pool, vec![row]).await?;

        results
            .into_iter()
            .next()
            .ok_or_else(|| ServiceError::NotFound("Task not found".into()))
    }

    pub async fn create(pool: &DbPool, user_id: i64, input: &CreateTask) -> ServiceResult<Task> {
        // Token parsing: extract #tags and @project from title
        let parsed = crate::filter::text_parser::parse_tokens(&input.title);

        let mut resolved_project_id = input.project_id;
        if let Some(ref project_name) = parsed.project {
            if let Ok(Some(pid)) =
                crate::ProjectService::find_by_title(pool, user_id, project_name).await
            {
                resolved_project_id = Some(pid);
            }
        }

        let title = if parsed.cleaned.is_empty() {
            input.title.clone()
        } else {
            parsed.cleaned
        };

        let mut conn = pool.get().await?;

        let sort_key = if let Some(ref sk) = input.sort_key {
            sk.clone()
        } else {
            let last_key: Option<String> = if input.parent_id.is_some() {
                tasks::table
                    .filter(tasks::parent_id.eq(input.parent_id))
                    .filter(tasks::user_id.eq(user_id))
                    .order(tasks::sort_key.desc())
                    .select(tasks::sort_key)
                    .first(&mut conn)
                    .await
                    .optional()?
            } else if resolved_project_id.is_some() {
                tasks::table
                    .filter(tasks::project_id.eq(resolved_project_id))
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
            north_dto::sort_key_after(last_key.as_deref())
        };

        let row = diesel::insert_into(tasks::table)
            .values(&NewTask {
                user_id,
                title: &title,
                body: input.body.as_deref(),
                project_id: resolved_project_id,
                parent_id: input.parent_id,
                sort_key: &sort_key,
                start_at: input.start_at,
                due_date: input.due_date,
                reviewed_at: input.reviewed_at,
                recurrence_type: None,
                recurrence_rule: None,
                is_url_fetching: None,
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

        // Add extracted tags
        if !parsed.tags.is_empty() {
            crate::TagService::add_task_tags_pooled(pool, user_id, task.id, &parsed.tags).await?;
            task = Self::get_by_id(pool, user_id, task.id).await?;
        }

        // Check for bare URLs and resolve in background
        task = Self::maybe_resolve_urls(pool, user_id, task).await?;

        Ok(task)
    }

    pub async fn update(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        input: &UpdateTask,
    ) -> ServiceResult<Task> {
        // Token parsing: extract #tags and @project from title if present
        let mut resolved_input = input.clone();
        let mut tags_to_add = Vec::new();
        let mut has_urls = false;

        if let Some(ref title) = input.title {
            let parsed = crate::filter::text_parser::parse_tokens(title);
            let cleaned = if parsed.cleaned.is_empty() {
                title.clone()
            } else {
                parsed.cleaned
            };
            if crate::url_service::has_bare_urls(&cleaned) {
                has_urls = true;
            }
            resolved_input.title = Some(cleaned);
            tags_to_add = parsed.tags;

            if let Some(ref project_name) = parsed.project {
                if let Ok(Some(pid)) =
                    crate::ProjectService::find_by_title(pool, user_id, project_name).await
                {
                    resolved_input.project_id = Some(Some(pid));
                }
            }
        }

        if let Some(Some(ref body)) = input.body {
            if crate::url_service::has_bare_urls(body) {
                has_urls = true;
            }
        }

        if has_urls {
            resolved_input.is_url_fetching = Some(Some(Utc::now()));
        }

        let mut task = Self::update_raw(pool, user_id, id, &resolved_input).await?;

        // Add extracted tags
        if !tags_to_add.is_empty() {
            crate::TagService::add_task_tags_pooled(pool, user_id, task.id, &tags_to_add).await?;
            task = Self::get_by_id(pool, user_id, task.id).await?;
        }

        // Resolve bare URLs in background
        if has_urls {
            let bg_pool = pool.clone();
            let task_id = task.id;
            let bg_title = task.title.clone();
            let bg_body = task.body.clone();
            tokio::spawn(async move {
                let resolved_title = crate::url_service::resolve_urls_in_text(&bg_title).await;
                let resolved_body = match bg_body {
                    Some(ref body) => Some(crate::url_service::resolve_urls_in_text(body).await),
                    None => None,
                };

                let update_input = UpdateTask {
                    title: Some(resolved_title),
                    body: Some(resolved_body),
                    is_url_fetching: Some(None),
                    ..Default::default()
                };
                if let Err(e) =
                    TaskService::update_raw(&bg_pool, user_id, task_id, &update_input).await
                {
                    tracing::error!(task_id, error = %e, "Background URL resolution failed");
                }
            });
        }

        Ok(task)
    }

    /// Raw update without token parsing — used by background URL resolution
    /// and `maybe_resolve_urls` to avoid re-parsing tokens.
    async fn update_raw(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        resolved_input: &UpdateTask,
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

        if let Some(ref title) = resolved_input.title {
            changeset.title = Some(title.as_str());
        }
        if let Some(ref body) = resolved_input.body {
            changeset.body = Some(body.as_deref());
        }
        if let Some(ref project_id) = resolved_input.project_id {
            changeset.project_id = Some(*project_id);
        }
        if let Some(ref parent_id) = resolved_input.parent_id {
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
        if let Some(ref sort_key) = resolved_input.sort_key {
            changeset.sort_key = Some(sort_key.as_str());
        }

        // When project or parent changes and no explicit sort_key was
        // provided, reset sort_key so the task lands at the bottom.
        let new_sort_key;
        let resolved_project = changeset.project_id.unwrap_or(existing.project_id);
        let resolved_parent = changeset.parent_id.unwrap_or(existing.parent_id);
        let project_changed =
            resolved_project != existing.project_id || resolved_parent != existing.parent_id;
        if project_changed && resolved_input.sort_key.is_none() {
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
            new_sort_key = north_dto::sort_key_after(last_key.as_deref());
            changeset.sort_key = Some(&new_sort_key);
        }
        if let Some(sequential_limit) = resolved_input.sequential_limit {
            changeset.sequential_limit = Some(sequential_limit);
        }
        if let Some(ref start_at) = resolved_input.start_at {
            changeset.start_at = Some(*start_at);
        }
        if let Some(ref due_date) = resolved_input.due_date {
            changeset.due_date = Some(*due_date);
        }
        if let Some(ref reviewed_at) = resolved_input.reviewed_at {
            changeset.reviewed_at = Some(*reviewed_at);
        }
        if let Some(ref completed_at) = resolved_input.completed_at {
            changeset.completed_at = Some(*completed_at);
        }
        if let Some(ref recurrence_type) = resolved_input.recurrence_type {
            changeset.recurrence_type = Some(recurrence_type.map(RecurrenceTypeMapping::from));
        }
        if let Some(ref recurrence_rule) = resolved_input.recurrence_rule {
            changeset.recurrence_rule = Some(recurrence_rule.as_deref());
        }
        if let Some(ref is_url_fetching) = resolved_input.is_url_fetching {
            changeset.is_url_fetching = Some(*is_url_fetching);
        }

        // When completing and no explicit sort_key, reset to empty.
        // When uncompleting and no explicit sort_key, place at end of list.
        let uncomplete_sort_key: String;
        if resolved_input.sort_key.is_none() && changeset.sort_key.is_none() {
            if let Some(Some(_)) = resolved_input.completed_at {
                if existing.completed_at.is_none() {
                    changeset.sort_key = Some("");
                }
            } else if let Some(None) = resolved_input.completed_at {
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
                    uncomplete_sort_key = north_dto::sort_key_after(last_key.as_deref());
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
        if let Some(Some(_)) = resolved_input.completed_at {
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
                    // Clear recurrence on children before completing
                    diesel::update(
                        tasks::table
                            .filter(tasks::id.eq_any(&child_ids))
                            .filter(tasks::recurrence_rule.is_not_null()),
                    )
                    .set((
                        tasks::recurrence_type.eq(None::<RecurrenceTypeMapping>),
                        tasks::recurrence_rule.eq(None::<String>),
                    ))
                    .execute(&mut conn)
                    .await?;

                    diesel::update(tasks::table.filter(tasks::id.eq_any(&child_ids)))
                        .set(tasks::completed_at.eq(Some(now)))
                        .execute(&mut conn)
                        .await?;
                    parent_ids = child_ids;
                }

                // Spawn next recurring instance if this task has recurrence
                if existing.recurrence_rule.is_some() {
                    let _ = Self::spawn_next_recurring(pool, user_id, &existing).await;
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

    // ── Recurrence ─────────────────────────────────────────────────

    async fn spawn_next_recurring(
        pool: &DbPool,
        user_id: i64,
        completed_task: &TaskRow,
    ) -> ServiceResult<Option<Task>> {
        let rec_rule = match completed_task.recurrence_rule.as_deref() {
            Some(r) if !r.is_empty() => r,
            _ => return Ok(None),
        };
        let rec_type = match completed_task.recurrence_type {
            Some(rt) => rt,
            None => return Ok(None),
        };

        // Load user settings for timezone
        let mut conn = pool.get().await?;
        let settings_val: serde_json::Value = users::table
            .filter(users::id.eq(user_id))
            .select(users::settings)
            .first(&mut conn)
            .await?;
        let settings: UserSettings = serde_json::from_value(settings_val).unwrap_or_default();
        let tz: chrono_tz::Tz = settings.timezone.parse().unwrap_or(chrono_tz::Tz::UTC);

        // Compute next occurrence
        let next_start = match RecurrenceType::from(rec_type) {
            RecurrenceType::Scheduled => Self::next_scheduled_date(
                rec_rule,
                completed_task.start_at,
                completed_task.due_date,
                tz,
            )?,
            RecurrenceType::AfterCompletion => Self::next_after_completion_date(rec_rule)?,
        };

        let Some(next_start) = next_start else {
            return Ok(None);
        };

        // Preserve due_date offset if original had both start_at and due_date
        let next_due = match (completed_task.start_at, completed_task.due_date) {
            (Some(orig_start), Some(orig_due)) => {
                let offset = orig_due.signed_duration_since(orig_start.date_naive());
                Some(next_start.date_naive() + offset)
            }
            (None, Some(orig_due)) => Some(orig_due),
            _ => None,
        };

        // Compute sort_key for the new task
        let last_key: Option<String> = if let Some(pid) = completed_task.parent_id {
            tasks::table
                .filter(tasks::parent_id.eq(pid))
                .filter(tasks::user_id.eq(user_id))
                .order(tasks::sort_key.desc())
                .select(tasks::sort_key)
                .first(&mut conn)
                .await
                .optional()?
        } else if let Some(proj) = completed_task.project_id {
            tasks::table
                .filter(tasks::project_id.eq(proj))
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
        let sort_key = north_dto::sort_key_after(last_key.as_deref());

        let new_row = diesel::insert_into(tasks::table)
            .values(&NewTask {
                user_id,
                title: &completed_task.title,
                body: completed_task.body.as_deref(),
                project_id: completed_task.project_id,
                parent_id: completed_task.parent_id,
                sort_key: &sort_key,
                start_at: Some(next_start),
                due_date: next_due,
                reviewed_at: None,
                recurrence_type: completed_task.recurrence_type,
                recurrence_rule: completed_task.recurrence_rule.as_deref(),
                is_url_fetching: None,
            })
            .returning(TaskRow::as_returning())
            .get_result(&mut conn)
            .await?;

        let new_task_id = new_row.id;

        // Clone subtasks from completed task
        let child_rows: Vec<TaskRow> = tasks::table
            .filter(tasks::parent_id.eq(completed_task.id))
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;

        for child in &child_rows {
            let child_sort_key = north_dto::sort_key_after(Some(&child.sort_key));
            let new_child = diesel::insert_into(tasks::table)
                .values(&NewTask {
                    user_id,
                    title: &child.title,
                    body: child.body.as_deref(),
                    project_id: child.project_id,
                    parent_id: Some(new_task_id),
                    sort_key: &child_sort_key,
                    start_at: None,
                    due_date: None,
                    reviewed_at: None,
                    recurrence_type: None,
                    recurrence_rule: None,
                    is_url_fetching: None,
                })
                .returning(TaskRow::as_returning())
                .get_result(&mut conn)
                .await?;

            // Copy tags from child
            let child_tag_ids: Vec<i64> = task_tags::table
                .filter(task_tags::task_id.eq(child.id))
                .select(task_tags::tag_id)
                .load(&mut conn)
                .await?;
            if !child_tag_ids.is_empty() {
                let links: Vec<NewTaskTag> = child_tag_ids
                    .iter()
                    .map(|&tag_id| NewTaskTag {
                        task_id: new_child.id,
                        tag_id,
                    })
                    .collect();
                diesel::insert_into(task_tags::table)
                    .values(&links)
                    .execute(&mut conn)
                    .await?;
            }
        }

        // Copy tags from parent task
        let parent_tag_ids: Vec<i64> = task_tags::table
            .filter(task_tags::task_id.eq(completed_task.id))
            .select(task_tags::tag_id)
            .load(&mut conn)
            .await?;
        if !parent_tag_ids.is_empty() {
            let links: Vec<NewTaskTag> = parent_tag_ids
                .iter()
                .map(|&tag_id| NewTaskTag {
                    task_id: new_task_id,
                    tag_id,
                })
                .collect();
            diesel::insert_into(task_tags::table)
                .values(&links)
                .execute(&mut conn)
                .await?;
        }

        Ok(Some(Task::from(new_row)))
    }

    fn next_scheduled_date(
        rrule_str: &str,
        start_at: Option<chrono::DateTime<Utc>>,
        due_date: Option<chrono::NaiveDate>,
        tz: chrono_tz::Tz,
    ) -> ServiceResult<Option<chrono::DateTime<Utc>>> {
        use chrono::TimeZone;

        let dt_start = start_at
            .or_else(|| due_date.map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap())))
            .unwrap_or_else(Utc::now);

        let rrule_tz: rrule::Tz = tz.into();
        let local_start = dt_start.with_timezone(&rrule_tz);
        let full_rule = format!(
            "DTSTART:{}\nRRULE:{rrule_str}",
            local_start.format("%Y%m%dT%H%M%S")
        );

        let rrule_set: rrule::RRuleSet = full_rule
            .parse()
            .map_err(|e| ServiceError::BadRequest(format!("Invalid RRULE: {e}")))?;

        let now_local = Utc::now().with_timezone(&rrule_tz);
        let results = rrule_set.after(now_local).all(1);

        Ok(results
            .dates
            .into_iter()
            .next()
            .map(|dt| dt.with_timezone(&Utc)))
    }

    fn next_after_completion_date(rrule_str: &str) -> ServiceResult<Option<chrono::DateTime<Utc>>> {
        let rule = match north_dto::RecurrenceRule::parse(rrule_str) {
            Some(r) => r,
            None => return Ok(None),
        };

        let interval = rule.interval as i64;
        let now = Utc::now();
        let next = match rule.freq {
            north_dto::Frequency::Daily => now + chrono::Duration::days(interval),
            north_dto::Frequency::Weekly => now + chrono::Duration::weeks(interval),
            north_dto::Frequency::Monthly => now + chrono::Duration::days(interval * 30),
            north_dto::Frequency::Yearly => now + chrono::Duration::days(interval * 365),
        };

        Ok(Some(next))
    }

    // ── Internal helpers ───────────────────────────────────────────

    async fn maybe_resolve_urls(
        pool: &DbPool,
        user_id: i64,
        mut task: Task,
    ) -> ServiceResult<Task> {
        let has_urls = crate::url_service::has_bare_urls(&task.title)
            || task
                .body
                .as_deref()
                .is_some_and(crate::url_service::has_bare_urls);

        if !has_urls {
            return Ok(task);
        }

        let flag_input = UpdateTask {
            is_url_fetching: Some(Some(Utc::now())),
            ..Default::default()
        };
        task = Self::update_raw(pool, user_id, task.id, &flag_input).await?;

        let bg_pool = pool.clone();
        let task_id = task.id;
        let bg_title = task.title.clone();
        let bg_body = task.body.clone();
        tokio::spawn(async move {
            let resolved_title = crate::url_service::resolve_urls_in_text(&bg_title).await;
            let resolved_body = match bg_body {
                Some(ref body) => Some(crate::url_service::resolve_urls_in_text(body).await),
                None => None,
            };

            let update_input = UpdateTask {
                title: Some(resolved_title),
                body: Some(resolved_body),
                is_url_fetching: Some(None),
                ..Default::default()
            };
            if let Err(e) = TaskService::update_raw(&bg_pool, user_id, task_id, &update_input).await
            {
                tracing::error!(task_id, error = %e, "Background URL resolution failed");
            }
        });

        Ok(task)
    }

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
                let mut task = Task::from(row);
                task.project_title = project_title;
                task.tags = tags;
                task.subtask_count = subtask_count;
                task.completed_subtask_count = completed_subtask_count;
                task
            })
            .collect())
    }

    pub async fn execute_dsl_filter(
        pool: &DbPool,
        user_id: i64,
        query_str: &str,
    ) -> ServiceResult<Vec<Task>> {
        let parsed = crate::filter::parse_filter(query_str).map_err(|errs| {
            ServiceError::BadRequest(
                errs.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;

        let mut conn = pool.get().await?;

        let matching_ids: Vec<i64> = if let Some(ref expr) = parsed.expression {
            let ids = crate::filter::eval_expr(pool, user_id, expr).await?;
            ids.into_iter().collect()
        } else {
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

        let rows: Vec<TaskRow> = tasks::table
            .filter(tasks::id.eq_any(&matching_ids))
            .select(TaskRow::as_select())
            .load(&mut conn)
            .await?;

        let mut results = Self::load_with_meta(pool, rows).await?;

        if let Some(ref order_by) = parsed.order_by {
            use crate::filter::dsl::{FilterField, SortDirection};
            results.sort_by(|a, b| {
                let cmp = match order_by.field {
                    FilterField::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                    FilterField::DueDate => a.due_date.cmp(&b.due_date),
                    FilterField::StartAt => a.start_at.cmp(&b.start_at),
                    FilterField::Created => a.created_at.cmp(&b.created_at),
                    FilterField::Updated => a.updated_at.cmp(&b.updated_at),
                    _ => a.sort_key.cmp(&b.sort_key),
                };
                match order_by.direction {
                    SortDirection::Asc => cmp,
                    SortDirection::Desc => cmp.reverse(),
                }
            });
        }

        Ok(results)
    }
}
