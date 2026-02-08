use axum::extract::{Path, State};
use axum::Json;
use north_domain::{
    Column, CreateColumn, CreateProject, Project, ProjectViewType,
    ProjectWithColumns, UpdateColumn, UpdateProject, UserSettings,
};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::AppState;

#[derive(Debug, sqlx::FromRow)]
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

#[derive(Debug, sqlx::FromRow)]
struct ColumnRow {
    id: i64,
    project_id: i64,
    name: String,
    color: String,
    position: i32,
    is_done: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ProjectRow> for Project {
    fn from(r: ProjectRow) -> Self {
        Project {
            id: r.id,
            user_id: r.user_id,
            title: r.title,
            description: r.description,
            view_type: parse_view_type(&r.view_type),
            position: r.position,
            archived: r.archived,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<ColumnRow> for Column {
    fn from(r: ColumnRow) -> Self {
        Column {
            id: r.id,
            project_id: r.project_id,
            name: r.name,
            color: r.color,
            position: r.position,
            is_done: r.is_done,
            created_at: r.created_at,
        }
    }
}

fn parse_view_type(s: &str) -> ProjectViewType {
    match s {
        "kanban" => ProjectViewType::Kanban,
        _ => ProjectViewType::List,
    }
}

fn view_type_to_str(vt: &ProjectViewType) -> &'static str {
    match vt {
        ProjectViewType::List => "list",
        ProjectViewType::Kanban => "kanban",
    }
}

pub async fn list_projects(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectWithColumns>>, AppError> {
    let projects = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, user_id, title, description, view_type::text as view_type, \
         position, archived, created_at, updated_at \
         FROM projects \
         WHERE user_id = $1 AND archived = false \
         ORDER BY position ASC",
    )
    .bind(auth_user.id)
    .fetch_all(&state.pool)
    .await?;

    let mut results = Vec::with_capacity(projects.len());
    for proj_row in projects {
        let columns = sqlx::query_as::<_, ColumnRow>(
            "SELECT id, project_id, name, color, position, is_done, \
             created_at \
             FROM project_columns \
             WHERE project_id = $1 \
             ORDER BY position ASC",
        )
        .bind(proj_row.id)
        .fetch_all(&state.pool)
        .await?;

        results.push(ProjectWithColumns {
            project: Project::from(proj_row),
            columns: columns.into_iter().map(Column::from).collect(),
        });
    }

    Ok(Json(results))
}

pub async fn create_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<CreateProject>,
) -> Result<Json<ProjectWithColumns>, AppError> {
    let view_type = body.view_type.as_ref().unwrap_or(&ProjectViewType::List);
    let vt_str = view_type_to_str(view_type);

    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM projects WHERE user_id = $1",
    )
    .bind(auth_user.id)
    .fetch_one(&state.pool)
    .await?;
    let position = max_pos.unwrap_or(-1) + 1;

    let proj_row = sqlx::query_as::<_, ProjectRow>(
        "INSERT INTO projects (user_id, title, description, view_type, position) \
         VALUES ($1, $2, $3, $4::project_view_type, $5) \
         RETURNING id, user_id, title, description, \
         view_type::text as view_type, position, archived, \
         created_at, updated_at",
    )
    .bind(auth_user.id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(vt_str)
    .bind(position)
    .fetch_one(&state.pool)
    .await?;

    // Fetch user settings for default columns
    let settings_val: serde_json::Value = sqlx::query_scalar(
        "SELECT settings FROM users WHERE id = $1",
    )
    .bind(auth_user.id)
    .fetch_one(&state.pool)
    .await?;
    let settings: UserSettings =
        serde_json::from_value(settings_val).unwrap_or_default();

    let mut columns = Vec::new();
    for (i, default_col) in settings.default_columns.iter().enumerate() {
        let col_row = sqlx::query_as::<_, ColumnRow>(
            "INSERT INTO project_columns \
             (project_id, name, color, position, is_done) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, project_id, name, color, position, \
             is_done, created_at",
        )
        .bind(proj_row.id)
        .bind(&default_col.name)
        .bind(&default_col.color)
        .bind(i as i32)
        .bind(default_col.is_done)
        .fetch_one(&state.pool)
        .await?;

        columns.push(Column::from(col_row));
    }

    Ok(Json(ProjectWithColumns {
        project: Project::from(proj_row),
        columns,
    }))
}

pub async fn get_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ProjectWithColumns>, AppError> {
    let proj_row = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, user_id, title, description, view_type::text as view_type, \
         position, archived, created_at, updated_at \
         FROM projects \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    let columns = sqlx::query_as::<_, ColumnRow>(
        "SELECT id, project_id, name, color, position, is_done, \
         created_at \
         FROM project_columns \
         WHERE project_id = $1 \
         ORDER BY position ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(ProjectWithColumns {
        project: Project::from(proj_row),
        columns: columns.into_iter().map(Column::from).collect(),
    }))
}

pub async fn update_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateProject>,
) -> Result<Json<Project>, AppError> {
    let existing = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, user_id, title, description, view_type::text as view_type, \
         position, archived, created_at, updated_at \
         FROM projects \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    let title = body.title.as_deref().unwrap_or(&existing.title);
    let description = body
        .description
        .as_ref()
        .or(existing.description.as_ref());
    let view_type = body
        .view_type
        .as_ref()
        .map(view_type_to_str)
        .unwrap_or(&existing.view_type);
    let position = body.position.unwrap_or(existing.position);
    let archived = body.archived.unwrap_or(existing.archived);

    let row = sqlx::query_as::<_, ProjectRow>(
        "UPDATE projects SET \
         title = $1, description = $2, \
         view_type = $3::project_view_type, \
         position = $4, archived = $5 \
         WHERE id = $6 AND user_id = $7 \
         RETURNING id, user_id, title, description, \
         view_type::text as view_type, position, archived, \
         created_at, updated_at",
    )
    .bind(title)
    .bind(description)
    .bind(view_type)
    .bind(position)
    .bind(archived)
    .bind(id)
    .bind(auth_user.id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(Project::from(row)))
}

pub async fn archive_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query(
        "UPDATE projects SET archived = true \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth_user.id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn create_column(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Json(body): Json<CreateColumn>,
) -> Result<Json<Column>, AppError> {
    // Verify project ownership
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM projects \
         WHERE id = $1 AND user_id = $2)",
    )
    .bind(project_id)
    .bind(auth_user.id)
    .fetch_one(&state.pool)
    .await?;

    if !exists {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM project_columns \
         WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_one(&state.pool)
    .await?;
    let position = max_pos.unwrap_or(-1) + 1;

    let color = body.color.as_deref().unwrap_or("#6b7280");
    let is_done = body.is_done.unwrap_or(false);

    let row = sqlx::query_as::<_, ColumnRow>(
        "INSERT INTO project_columns \
         (project_id, name, color, position, is_done) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, project_id, name, color, position, \
         is_done, created_at",
    )
    .bind(project_id)
    .bind(&body.name)
    .bind(color)
    .bind(position)
    .bind(is_done)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(Column::from(row)))
}

pub async fn update_column(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(column_id): Path<i64>,
    Json(body): Json<UpdateColumn>,
) -> Result<Json<Column>, AppError> {
    let existing = sqlx::query_as::<_, ColumnRow>(
        "SELECT pc.id, pc.project_id, pc.name, pc.color, pc.position, \
         pc.is_done, pc.created_at \
         FROM project_columns pc \
         JOIN projects p ON p.id = pc.project_id \
         WHERE pc.id = $1 AND p.user_id = $2",
    )
    .bind(column_id)
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    let name = body.name.as_deref().unwrap_or(&existing.name);
    let color = body.color.as_deref().unwrap_or(&existing.color);
    let position = body.position.unwrap_or(existing.position);
    let is_done = body.is_done.unwrap_or(existing.is_done);

    let row = sqlx::query_as::<_, ColumnRow>(
        "UPDATE project_columns SET \
         name = $1, color = $2, position = $3, is_done = $4 \
         WHERE id = $5 \
         RETURNING id, project_id, name, color, position, \
         is_done, created_at",
    )
    .bind(name)
    .bind(color)
    .bind(position)
    .bind(is_done)
    .bind(column_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(Column::from(row)))
}

pub async fn delete_column(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(column_id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    // Verify ownership and get column info
    let existing = sqlx::query_as::<_, ColumnRow>(
        "SELECT pc.id, pc.project_id, pc.name, pc.color, pc.position, \
         pc.is_done, pc.created_at \
         FROM project_columns pc \
         JOIN projects p ON p.id = pc.project_id \
         WHERE pc.id = $1 AND p.user_id = $2",
    )
    .bind(column_id)
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Column not found".to_string()))?;

    // Check if tasks use this column, reassign to first column if so
    let task_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE column_id = $1",
    )
    .bind(column_id)
    .fetch_one(&state.pool)
    .await?;

    if task_count > 0 {
        let first_column: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM project_columns \
             WHERE project_id = $1 AND id != $2 \
             ORDER BY position ASC LIMIT 1",
        )
        .bind(existing.project_id)
        .bind(column_id)
        .fetch_optional(&state.pool)
        .await?;

        match first_column {
            Some(new_col_id) => {
                sqlx::query(
                    "UPDATE tasks SET column_id = $1 \
                     WHERE column_id = $2",
                )
                .bind(new_col_id)
                .bind(column_id)
                .execute(&state.pool)
                .await?;
            }
            None => {
                return Err(AppError::BadRequest(
                    "Cannot delete the only column while tasks \
                     are assigned to it"
                        .to_string(),
                ));
            }
        }
    }

    sqlx::query("DELETE FROM project_columns WHERE id = $1")
        .bind(column_id)
        .execute(&state.pool)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
