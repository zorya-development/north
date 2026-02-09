use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use north_domain::{CreateTask, MoveTask, Task, TaskFilter, TaskWithMeta, UpdateTask};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::AppState;

#[derive(Debug, sqlx::FromRow)]
struct TaskRow {
    id: i64,
    project_id: Option<i64>,
    parent_id: Option<i64>,
    column_id: Option<i64>,
    user_id: i64,
    title: String,
    body: Option<String>,
    position: i32,
    sequential_limit: i16,
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    due_date: Option<chrono::NaiveDate>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    reviewed_at: Option<chrono::NaiveDate>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct TaskWithMetaRow {
    id: i64,
    project_id: Option<i64>,
    parent_id: Option<i64>,
    column_id: Option<i64>,
    user_id: i64,
    title: String,
    body: Option<String>,
    position: i32,
    sequential_limit: i16,
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    due_date: Option<chrono::NaiveDate>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    reviewed_at: Option<chrono::NaiveDate>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    project_title: Option<String>,
    column_name: Option<String>,
    tags: Option<String>,
    subtask_count: i64,
    actionable: bool,
}

impl From<TaskRow> for Task {
    fn from(r: TaskRow) -> Self {
        Task {
            id: r.id,
            project_id: r.project_id,
            parent_id: r.parent_id,
            column_id: r.column_id,
            user_id: r.user_id,
            title: r.title,
            body: r.body,
            position: r.position,
            sequential_limit: r.sequential_limit,
            start_at: r.start_at,
            due_date: r.due_date,
            completed_at: r.completed_at,
            reviewed_at: r.reviewed_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<TaskWithMetaRow> for TaskWithMeta {
    fn from(r: TaskWithMetaRow) -> Self {
        let tags = r
            .tags
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        TaskWithMeta {
            task: Task {
                id: r.id,
                project_id: r.project_id,
                parent_id: r.parent_id,
                column_id: r.column_id,
                user_id: r.user_id,
                title: r.title,
                body: r.body,
                position: r.position,
                sequential_limit: r.sequential_limit,
                start_at: r.start_at,
                due_date: r.due_date,
                completed_at: r.completed_at,
                reviewed_at: r.reviewed_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            project_title: r.project_title,
            column_name: r.column_name,
            tags,
            subtask_count: r.subtask_count,
            actionable: r.actionable,
        }
    }
}

pub async fn list_tasks(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Query(filter): Query<TaskFilter>,
) -> Result<Json<Vec<TaskWithMeta>>, AppError> {
    let mut conditions = vec!["t.user_id = $1".to_string()];
    let mut param_idx: usize = 2;

    // Build dynamic filter conditions
    let mut bind_values: Vec<BindValue> = vec![];

    if let Some(project_id) = filter.project {
        conditions.push(format!("t.project_id = ${param_idx}"));
        bind_values.push(BindValue::BigInt(project_id));
        param_idx += 1;
    }

    if let Some(parent_id) = filter.parent {
        conditions.push(format!("t.parent_id = ${param_idx}"));
        bind_values.push(BindValue::BigInt(parent_id));
        param_idx += 1;
    }

    if let Some(column_id) = filter.column {
        conditions.push(format!("t.column_id = ${param_idx}"));
        bind_values.push(BindValue::BigInt(column_id));
        param_idx += 1;
    }

    if let Some(ref tags) = filter.tag {
        if !tags.is_empty() {
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM task_tags tt \
                 JOIN tags tg ON tg.id = tt.tag_id \
                 WHERE tt.task_id = t.id AND tg.name = ANY(${param_idx}))"
            ));
            bind_values.push(BindValue::StringVec(tags.clone()));
            param_idx += 1;
        }
    }

    if filter.inbox == Some(true) {
        conditions.push("t.project_id IS NULL".to_string());
        conditions.push("t.parent_id IS NULL".to_string());
    }

    if filter.completed == Some(true) {
        conditions.push("t.completed_at IS NOT NULL".to_string());
    } else if filter.completed == Some(false) {
        conditions.push("t.completed_at IS NULL".to_string());
    }

    if let Some(ref q) = filter.q {
        conditions.push(format!(
            "(t.title ILIKE ${param_idx} OR t.body ILIKE ${param_idx})"
        ));
        bind_values.push(BindValue::Text(format!("%{q}%")));
        param_idx += 1;
    }

    if filter.review_due == Some(true) {
        conditions.push(
            "(t.reviewed_at IS NULL OR t.reviewed_at <= \
             CURRENT_DATE - (SELECT (u.settings->>'review_interval_days')::int \
             FROM users u WHERE u.id = t.user_id) * INTERVAL '1 day')"
                .to_string(),
        );
        conditions.push("t.completed_at IS NULL".to_string());
    }

    let where_clause = conditions.join(" AND ");

    let order_clause = match filter.sort.as_deref() {
        Some("due_date") => "t.due_date ASC NULLS LAST, t.position ASC",
        Some("created_at") => "t.created_at DESC",
        Some("title") => "t.title ASC",
        _ => "t.position ASC, t.created_at DESC",
    };

    let limit_clause = if let Some(limit) = filter.limit {
        bind_values.push(BindValue::BigInt(limit));
        let offset_part = if let Some(offset) = filter.offset {
            bind_values.push(BindValue::BigInt(offset));
            format!(" LIMIT ${} OFFSET ${}", param_idx, param_idx + 1)
        } else {
            let s = format!(" LIMIT ${param_idx}");
            s
        };
        param_idx += if filter.offset.is_some() { 2 } else { 1 };
        offset_part
    } else {
        String::new()
    };
    let _ = param_idx;

    // Build the query with actionable computation
    let sql = format!(
        "SELECT \
            t.id, t.project_id, t.parent_id, t.column_id, t.user_id, \
            t.title, t.body, t.position, t.sequential_limit, \
            t.start_at, t.due_date, t.completed_at, t.reviewed_at, \
            t.created_at, t.updated_at, \
            p.title as project_title, \
            pc.name as column_name, \
            (SELECT string_agg(tg.name, ',') \
             FROM task_tags tt JOIN tags tg ON tg.id = tt.tag_id \
             WHERE tt.task_id = t.id) as tags, \
            (SELECT COUNT(*) FROM tasks sub WHERE sub.parent_id = t.id) \
            as subtask_count, \
            CASE WHEN t.completed_at IS NOT NULL THEN false \
                 WHEN t.start_at IS NOT NULL \
                      AND t.start_at::date > CURRENT_DATE THEN false \
                 WHEN t.parent_id IS NULL THEN true \
                 ELSE ( \
                    ROW_NUMBER() OVER ( \
                        PARTITION BY t.parent_id \
                        ORDER BY t.position \
                    ) <= COALESCE( \
                        (SELECT pt.sequential_limit \
                         FROM tasks pt WHERE pt.id = t.parent_id), 1) \
                 ) \
            END as actionable \
        FROM tasks t \
        LEFT JOIN projects p ON p.id = t.project_id \
        LEFT JOIN project_columns pc ON pc.id = t.column_id \
        WHERE {where_clause} \
        ORDER BY {order_clause} \
        {limit_clause}"
    );

    let mut query = sqlx::query_as::<_, TaskWithMetaRow>(&sql).bind(auth_user.id);

    for val in &bind_values {
        match val {
            BindValue::BigInt(v) => query = query.bind(*v),
            BindValue::Text(v) => query = query.bind(v.as_str()),
            BindValue::StringVec(v) => query = query.bind(v),
        }
    }

    let rows = query.fetch_all(&state.pool).await?;

    let mut results: Vec<TaskWithMeta> = rows.into_iter().map(TaskWithMeta::from).collect();

    // Post-filter actionable if requested
    if filter.actionable == Some(true) {
        results.retain(|t| t.actionable);
    }

    Ok(Json(results))
}

pub async fn create_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<CreateTask>,
) -> Result<Json<Task>, AppError> {
    let max_pos: Option<i32> = if body.parent_id.is_some() {
        sqlx::query_scalar(
            "SELECT MAX(position) FROM tasks \
             WHERE parent_id = $1 AND user_id = $2",
        )
        .bind(body.parent_id)
        .bind(auth_user.id)
        .fetch_one(&state.pool)
        .await?
    } else if body.project_id.is_some() {
        sqlx::query_scalar(
            "SELECT MAX(position) FROM tasks \
             WHERE project_id = $1 AND parent_id IS NULL AND user_id = $2",
        )
        .bind(body.project_id)
        .bind(auth_user.id)
        .fetch_one(&state.pool)
        .await?
    } else {
        sqlx::query_scalar(
            "SELECT MAX(position) FROM tasks \
             WHERE project_id IS NULL AND parent_id IS NULL \
             AND user_id = $1",
        )
        .bind(auth_user.id)
        .fetch_one(&state.pool)
        .await?
    };

    let position = max_pos.unwrap_or(-1) + 1;

    let row = sqlx::query_as::<_, TaskRow>(
        "INSERT INTO tasks \
         (project_id, parent_id, column_id, user_id, title, body, \
          position, start_at, due_date) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         RETURNING id, project_id, parent_id, column_id, user_id, \
         title, body, position, sequential_limit, start_at, due_date, \
         completed_at, reviewed_at, created_at, updated_at",
    )
    .bind(body.project_id)
    .bind(body.parent_id)
    .bind(body.column_id)
    .bind(auth_user.id)
    .bind(&body.title)
    .bind(&body.body)
    .bind(position)
    .bind(body.start_at)
    .bind(body.due_date)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(Task::from(row)))
}

pub async fn get_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<TaskWithMeta>, AppError> {
    let row = sqlx::query_as::<_, TaskWithMetaRow>(
        "SELECT \
            t.id, t.project_id, t.parent_id, t.column_id, t.user_id, \
            t.title, t.body, t.position, t.sequential_limit, \
            t.start_at, t.due_date, t.completed_at, t.reviewed_at, \
            t.created_at, t.updated_at, \
            p.title as project_title, \
            pc.name as column_name, \
            (SELECT string_agg(tg.name, ',') \
             FROM task_tags tt JOIN tags tg ON tg.id = tt.tag_id \
             WHERE tt.task_id = t.id) as tags, \
            (SELECT COUNT(*) FROM tasks sub WHERE sub.parent_id = t.id) \
            as subtask_count, \
            CASE WHEN t.completed_at IS NOT NULL THEN false \
                 WHEN t.start_at IS NOT NULL \
                      AND t.start_at::date > CURRENT_DATE THEN false \
                 WHEN t.parent_id IS NULL THEN true \
                 ELSE ( \
                    (SELECT COUNT(*) FROM tasks sib \
                     WHERE sib.parent_id = t.parent_id \
                     AND sib.completed_at IS NULL \
                     AND sib.position < t.position) \
                    < COALESCE( \
                        (SELECT pt.sequential_limit \
                         FROM tasks pt WHERE pt.id = t.parent_id), 1) \
                 ) \
            END as actionable \
        FROM tasks t \
        LEFT JOIN projects p ON p.id = t.project_id \
        LEFT JOIN project_columns pc ON pc.id = t.column_id \
        WHERE t.id = $1 AND t.user_id = $2",
    )
    .bind(id)
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(TaskWithMeta::from(row)))
}

pub async fn update_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTask>,
) -> Result<Json<Task>, AppError> {
    let existing = sqlx::query_as::<_, TaskRow>(
        "SELECT id, project_id, parent_id, column_id, user_id, \
         title, body, position, sequential_limit, start_at, due_date, \
         completed_at, reviewed_at, created_at, updated_at \
         FROM tasks WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    let title = body.title.as_deref().unwrap_or(&existing.title);
    let body_text = body.body.as_ref().or(existing.body.as_ref());
    let project_id = body.project_id.or(existing.project_id);
    let parent_id = body.parent_id.or(existing.parent_id);
    let new_column_id = body.column_id.or(existing.column_id);
    let position = body.position.unwrap_or(existing.position);
    let sequential_limit = body.sequential_limit.unwrap_or(existing.sequential_limit);
    let start_at = body.start_at.or(existing.start_at);
    let due_date = body.due_date.or(existing.due_date);

    // Handle completed_at based on column change
    let completed_at = if body.column_id.is_some() && body.column_id != existing.column_id {
        if let Some(col_id) = new_column_id {
            let is_done: Option<bool> =
                sqlx::query_scalar("SELECT is_done FROM project_columns WHERE id = $1")
                    .bind(col_id)
                    .fetch_optional(&state.pool)
                    .await?;

            if is_done == Some(true) {
                Some(Utc::now())
            } else {
                None
            }
        } else {
            existing.completed_at
        }
    } else {
        existing.completed_at
    };

    let row = sqlx::query_as::<_, TaskRow>(
        "UPDATE tasks SET \
         title = $1, body = $2, project_id = $3, parent_id = $4, \
         column_id = $5, position = $6, sequential_limit = $7, \
         start_at = $8, due_date = $9, completed_at = $10 \
         WHERE id = $11 AND user_id = $12 \
         RETURNING id, project_id, parent_id, column_id, user_id, \
         title, body, position, sequential_limit, start_at, due_date, \
         completed_at, reviewed_at, created_at, updated_at",
    )
    .bind(title)
    .bind(body_text)
    .bind(project_id)
    .bind(parent_id)
    .bind(new_column_id)
    .bind(position)
    .bind(sequential_limit)
    .bind(start_at)
    .bind(due_date)
    .bind(completed_at)
    .bind(id)
    .bind(auth_user.id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(Task::from(row)))
}

pub async fn delete_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth_user.id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Task not found".to_string()));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn move_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<MoveTask>,
) -> Result<Json<Task>, AppError> {
    let existing = sqlx::query_as::<_, TaskRow>(
        "SELECT id, project_id, parent_id, column_id, user_id, \
         title, body, position, sequential_limit, start_at, due_date, \
         completed_at, reviewed_at, created_at, updated_at \
         FROM tasks WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    let new_column_id = body.column_id.or(existing.column_id);
    let new_position = body.position.unwrap_or(existing.position);
    let new_parent_id = body.parent_id.or(existing.parent_id);

    // Handle completed_at based on column change
    let completed_at = if body.column_id.is_some() && body.column_id != existing.column_id {
        if let Some(col_id) = new_column_id {
            let is_done: Option<bool> =
                sqlx::query_scalar("SELECT is_done FROM project_columns WHERE id = $1")
                    .bind(col_id)
                    .fetch_optional(&state.pool)
                    .await?;

            if is_done == Some(true) {
                Some(Utc::now())
            } else {
                None
            }
        } else {
            existing.completed_at
        }
    } else {
        existing.completed_at
    };

    let row = sqlx::query_as::<_, TaskRow>(
        "UPDATE tasks SET \
         column_id = $1, position = $2, parent_id = $3, completed_at = $4 \
         WHERE id = $5 AND user_id = $6 \
         RETURNING id, project_id, parent_id, column_id, user_id, \
         title, body, position, sequential_limit, start_at, due_date, \
         completed_at, reviewed_at, created_at, updated_at",
    )
    .bind(new_column_id)
    .bind(new_position)
    .bind(new_parent_id)
    .bind(completed_at)
    .bind(id)
    .bind(auth_user.id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(Task::from(row)))
}

pub async fn review_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Task>, AppError> {
    let row = sqlx::query_as::<_, TaskRow>(
        "UPDATE tasks SET reviewed_at = CURRENT_DATE \
         WHERE id = $1 AND user_id = $2 \
         RETURNING id, project_id, parent_id, column_id, user_id, \
         title, body, position, sequential_limit, start_at, due_date, \
         completed_at, reviewed_at, created_at, updated_at",
    )
    .bind(id)
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(Task::from(row)))
}

enum BindValue {
    BigInt(i64),
    Text(String),
    StringVec(Vec<String>),
}
