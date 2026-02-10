use axum::extract::{Path, State};
use axum::Json;
use north_domain::{
    Column, CreateColumn, CreateProject, Project, ProjectWithColumns, UpdateColumn, UpdateProject,
};
use north_services::{ColumnService, ProjectService};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::AppState;

pub async fn list_projects(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectWithColumns>>, AppError> {
    let results = ProjectService::list_with_columns(&state.pool, auth_user.id).await?;
    Ok(Json(results))
}

pub async fn create_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<CreateProject>,
) -> Result<Json<ProjectWithColumns>, AppError> {
    let result = ProjectService::create(&state.pool, auth_user.id, &body).await?;
    Ok(Json(result))
}

pub async fn get_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ProjectWithColumns>, AppError> {
    let result = ProjectService::get_with_columns(&state.pool, auth_user.id, id).await?;
    Ok(Json(result))
}

pub async fn update_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateProject>,
) -> Result<Json<Project>, AppError> {
    let result = ProjectService::update(&state.pool, auth_user.id, id, &body).await?;
    Ok(Json(result))
}

pub async fn archive_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    ProjectService::archive(&state.pool, auth_user.id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn create_column(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Json(body): Json<CreateColumn>,
) -> Result<Json<Column>, AppError> {
    let result = ColumnService::create(&state.pool, auth_user.id, project_id, &body).await?;
    Ok(Json(result))
}

pub async fn update_column(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(column_id): Path<i64>,
    Json(body): Json<UpdateColumn>,
) -> Result<Json<Column>, AppError> {
    let result = ColumnService::update(&state.pool, auth_user.id, column_id, &body).await?;
    Ok(Json(result))
}

pub async fn delete_column(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(column_id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    ColumnService::delete(&state.pool, auth_user.id, column_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
