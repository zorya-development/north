use leptos::prelude::*;
use north_domain::{Task, TaskWithMeta};

// ── Query server functions ──────────────────────────────────────────

#[server(GetInboxTasksFn, "/api")]
pub async fn get_inbox_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let rows = sqlx::query_as::<_, FullTaskRow>(
        "SELECT t.id, t.project_id, t.parent_id, t.column_id, t.user_id, \
         t.title, t.body, t.position, t.sequential_limit, \
         t.start_at, t.due_date, t.completed_at, t.reviewed_at, \
         t.created_at, t.updated_at, \
         NULL::text as project_title, \
         NULL::text as column_name, \
         (SELECT count(*) FROM tasks s WHERE s.parent_id = t.id) \
             as subtask_count, \
         (SELECT json_agg(tg.name) FROM task_tags tt \
          JOIN tags tg ON tg.id = tt.tag_id WHERE tt.task_id = t.id) as tags \
         FROM tasks t \
         WHERE t.project_id IS NULL \
           AND t.parent_id IS NULL \
           AND t.user_id = $1 \
           AND t.completed_at IS NULL \
         ORDER BY t.position",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows.into_iter().map(Into::into).collect())
}

#[server(GetTodayTasksFn, "/api")]
pub async fn get_today_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let rows = sqlx::query_as::<_, FullTaskRow>(
        "SELECT t.id, t.project_id, t.parent_id, t.column_id, t.user_id, \
         t.title, t.body, t.position, t.sequential_limit, \
         t.start_at, t.due_date, t.completed_at, t.reviewed_at, \
         t.created_at, t.updated_at, \
         p.title as project_title, \
         pc.name as column_name, \
         (SELECT count(*) FROM tasks s WHERE s.parent_id = t.id) \
             as subtask_count, \
         (SELECT json_agg(tg.name) FROM task_tags tt \
          JOIN tags tg ON tg.id = tt.tag_id \
          WHERE tt.task_id = t.id) as tags \
         FROM tasks t \
         LEFT JOIN projects p ON p.id = t.project_id \
         LEFT JOIN project_columns pc ON pc.id = t.column_id \
         WHERE t.user_id = $1 \
           AND t.start_at IS NOT NULL \
           AND t.start_at::date <= CURRENT_DATE \
           AND t.completed_at IS NULL \
         ORDER BY t.start_at ASC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows.into_iter().map(Into::into).collect())
}

#[server(GetAllTasksFn, "/api")]
pub async fn get_all_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let rows = sqlx::query_as::<_, FullTaskRow>(
        "SELECT t.id, t.project_id, t.parent_id, t.column_id, t.user_id, \
         t.title, t.body, t.position, t.sequential_limit, \
         t.start_at, t.due_date, t.completed_at, t.reviewed_at, \
         t.created_at, t.updated_at, \
         p.title as project_title, \
         pc.name as column_name, \
         (SELECT count(*) FROM tasks s WHERE s.parent_id = t.id) \
             as subtask_count, \
         (SELECT json_agg(tg.name) FROM task_tags tt \
          JOIN tags tg ON tg.id = tt.tag_id \
          WHERE tt.task_id = t.id) as tags \
         FROM tasks t \
         LEFT JOIN projects p ON p.id = t.project_id \
         LEFT JOIN project_columns pc ON pc.id = t.column_id \
         WHERE t.parent_id IS NULL \
           AND t.user_id = $1 \
         ORDER BY \
           CASE WHEN t.completed_at IS NULL THEN 0 ELSE 1 END, \
           t.position ASC, \
           t.completed_at DESC NULLS FIRST",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let completed = row.completed_at.is_some();
            let mut meta: TaskWithMeta = row.into();
            meta.actionable = !completed;
            meta
        })
        .collect())
}

// ── Mutation server functions ───────────────────────────────────────

#[server(CreateTaskFn, "/api")]
pub async fn create_task(title: String, body: Option<String>) -> Result<Task, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM tasks \
         WHERE user_id = $1 AND project_id IS NULL AND parent_id IS NULL",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let position = max_pos.unwrap_or(0) + 1;
    let body = body.filter(|b| !b.trim().is_empty());

    #[derive(sqlx::FromRow)]
    struct InsertedTask {
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
        reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let row = sqlx::query_as::<_, InsertedTask>(
        "INSERT INTO tasks (user_id, title, body, position, sequential_limit) \
         VALUES ($1, $2, $3, $4, 1) \
         RETURNING id, project_id, parent_id, column_id, user_id, \
         title, body, position, sequential_limit, \
         start_at, due_date, completed_at, reviewed_at, \
         created_at, updated_at",
    )
    .bind(user_id)
    .bind(&title)
    .bind(&body)
    .bind(position)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(Task {
        id: row.id,
        project_id: row.project_id,
        parent_id: row.parent_id,
        column_id: row.column_id,
        user_id: row.user_id,
        title: row.title,
        body: row.body,
        position: row.position,
        sequential_limit: row.sequential_limit,
        start_at: row.start_at,
        due_date: row.due_date,
        completed_at: row.completed_at,
        reviewed_at: row.reviewed_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

#[server(UpdateTaskFn, "/api")]
pub async fn update_task(
    id: i64,
    title: String,
    body: Option<String>,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    let body = body.filter(|b| !b.trim().is_empty());

    let result = sqlx::query(
        "UPDATE tasks SET title = $1, body = $2, updated_at = now() \
         WHERE id = $3 AND user_id = $4",
    )
    .bind(&title)
    .bind(&body)
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[server(CompleteTaskFn, "/api")]
pub async fn complete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE tasks SET completed_at = now() \
         WHERE id = $1 AND user_id = $2 AND completed_at IS NULL",
    )
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[server(UncompleteTaskFn, "/api")]
pub async fn uncomplete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE tasks SET completed_at = NULL \
         WHERE id = $1 AND user_id = $2 AND completed_at IS NOT NULL",
    )
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[server(DeleteTaskFn, "/api")]
pub async fn delete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[server(SetTaskStartAtFn, "/api")]
pub async fn set_task_start_at(id: i64, start_at: String) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let dt = chrono::NaiveDateTime::parse_from_str(&start_at, "%Y-%m-%dT%H:%M")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&start_at, "%Y-%m-%dT%H:%M:%S"))
        .map_err(|e| ServerFnError::new(format!("Invalid datetime: {e}")))?;

    let dt_utc = dt.and_utc();

    let result = sqlx::query(
        "UPDATE tasks SET start_at = $1 \
         WHERE id = $2 AND user_id = $3",
    )
    .bind(dt_utc)
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[server(ClearTaskStartAtFn, "/api")]
pub async fn clear_task_start_at(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE tasks SET start_at = NULL \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

// ── Shared row type ─────────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct FullTaskRow {
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
    reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    project_title: Option<String>,
    column_name: Option<String>,
    subtask_count: Option<i64>,
    tags: Option<serde_json::Value>,
}

#[cfg(feature = "ssr")]
impl From<FullTaskRow> for TaskWithMeta {
    fn from(row: FullTaskRow) -> Self {
        let tags: Vec<String> = row
            .tags
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        TaskWithMeta {
            task: Task {
                id: row.id,
                project_id: row.project_id,
                parent_id: row.parent_id,
                column_id: row.column_id,
                user_id: row.user_id,
                title: row.title,
                body: row.body,
                position: row.position,
                sequential_limit: row.sequential_limit,
                start_at: row.start_at,
                due_date: row.due_date,
                completed_at: row.completed_at,
                reviewed_at: row.reviewed_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            project_title: row.project_title,
            column_name: row.column_name,
            tags,
            subtask_count: row.subtask_count.unwrap_or(0),
            actionable: true,
        }
    }
}
