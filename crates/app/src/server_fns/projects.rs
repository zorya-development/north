use leptos::prelude::*;
use north_domain::{Project, TaskWithMeta};

#[server(GetProjectsFn, "/api")]
pub async fn get_projects() -> Result<Vec<Project>, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    #[derive(sqlx::FromRow)]
    struct ProjectRow {
        id: i64,
        user_id: i64,
        title: String,
        description: Option<String>,
        view_type: String,
        position: i32,
        archived: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let rows = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, user_id, title, description, \
         view_type::text, position, archived, created_at, updated_at \
         FROM projects \
         WHERE user_id = $1 AND archived = false \
         ORDER BY position ASC, created_at ASC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| Project {
            id: r.id,
            user_id: r.user_id,
            title: r.title,
            description: r.description,
            view_type: match r.view_type.as_str() {
                "kanban" => north_domain::ProjectViewType::Kanban,
                _ => north_domain::ProjectViewType::List,
            },
            position: r.position,
            archived: r.archived,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

#[server(CreateProjectFn, "/api")]
pub async fn create_project(title: String) -> Result<Project, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM projects WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let position = max_pos.unwrap_or(0) + 1;

    #[derive(sqlx::FromRow)]
    struct InsertedRow {
        id: i64,
        user_id: i64,
        title: String,
        description: Option<String>,
        view_type: String,
        position: i32,
        archived: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let row = sqlx::query_as::<_, InsertedRow>(
        "INSERT INTO projects (user_id, title, position) \
         VALUES ($1, $2, $3) \
         RETURNING id, user_id, title, description, \
         view_type::text, position, archived, created_at, updated_at",
    )
    .bind(user_id)
    .bind(&title)
    .bind(position)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(Project {
        id: row.id,
        user_id: row.user_id,
        title: row.title,
        description: row.description,
        view_type: match row.view_type.as_str() {
            "kanban" => north_domain::ProjectViewType::Kanban,
            _ => north_domain::ProjectViewType::List,
        },
        position: row.position,
        archived: row.archived,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

#[server(SetTaskProjectFn, "/api")]
pub async fn set_task_project(task_id: i64, project_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE tasks SET project_id = $1, updated_at = now() \
         WHERE id = $2 AND user_id = $3",
    )
    .bind(project_id)
    .bind(task_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[server(ClearTaskProjectFn, "/api")]
pub async fn clear_task_project(task_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE tasks SET project_id = NULL, updated_at = now() \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(task_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[server(ArchiveProjectFn, "/api")]
pub async fn archive_project(project_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE projects SET archived = true, updated_at = now() \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Project not found".to_string()));
    }

    Ok(())
}

#[server(UnarchiveProjectFn, "/api")]
pub async fn unarchive_project(project_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE projects SET archived = false, updated_at = now() \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Project not found".to_string()));
    }

    Ok(())
}

#[server(GetProjectFn, "/api")]
pub async fn get_project(project_id: i64) -> Result<Project, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    #[derive(sqlx::FromRow)]
    struct ProjectRow {
        id: i64,
        user_id: i64,
        title: String,
        description: Option<String>,
        view_type: String,
        position: i32,
        archived: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let row = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, user_id, title, description, \
         view_type::text, position, archived, created_at, updated_at \
         FROM projects \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Project not found".to_string()))?;

    Ok(Project {
        id: row.id,
        user_id: row.user_id,
        title: row.title,
        description: row.description,
        view_type: match row.view_type.as_str() {
            "kanban" => north_domain::ProjectViewType::Kanban,
            _ => north_domain::ProjectViewType::List,
        },
        position: row.position,
        archived: row.archived,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

#[server(GetProjectTasksFn, "/api")]
pub async fn get_project_tasks(
    project_id: i64,
) -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

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
         WHERE t.project_id = $1 \
           AND t.user_id = $2 \
           AND t.parent_id IS NULL \
           AND t.completed_at IS NULL \
         ORDER BY t.position ASC",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let tags: Vec<String> = row
                .tags
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            TaskWithMeta {
                task: north_domain::Task {
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
        })
        .collect())
}

#[server(DeleteProjectFn, "/api")]
pub async fn delete_project(project_id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "DELETE FROM projects WHERE id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Project not found".to_string()));
    }

    Ok(())
}

#[server(GetArchivedProjectsFn, "/api")]
pub async fn get_archived_projects() -> Result<Vec<Project>, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    #[derive(sqlx::FromRow)]
    struct ProjectRow {
        id: i64,
        user_id: i64,
        title: String,
        description: Option<String>,
        view_type: String,
        position: i32,
        archived: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let rows = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, user_id, title, description, \
         view_type::text, position, archived, created_at, updated_at \
         FROM projects \
         WHERE user_id = $1 AND archived = true \
         ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| Project {
            id: r.id,
            user_id: r.user_id,
            title: r.title,
            description: r.description,
            view_type: match r.view_type.as_str() {
                "kanban" => north_domain::ProjectViewType::Kanban,
                _ => north_domain::ProjectViewType::List,
            },
            position: r.position,
            archived: r.archived,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}
