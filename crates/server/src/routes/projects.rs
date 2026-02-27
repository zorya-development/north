use axum::extract::{Path, Query, State};
use axum::Json;
use north_core::ProjectService;
use north_dto::{CreateProject, Project, ProjectFilter, UpdateProject};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::AppState;

pub async fn list_projects(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Query(filter): Query<ProjectFilter>,
) -> Result<Json<Vec<Project>>, AppError> {
    let results = ProjectService::list(&state.pool, auth_user.id, &filter).await?;
    Ok(Json(results))
}

pub async fn create_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<CreateProject>,
) -> Result<Json<Project>, AppError> {
    let result = ProjectService::create(&state.pool, auth_user.id, &body).await?;
    Ok(Json(result))
}

pub async fn get_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Project>, AppError> {
    let result = ProjectService::get_by_id(&state.pool, auth_user.id, id).await?;
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

pub async fn delete_project(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    ProjectService::delete(&state.pool, auth_user.id, id).await?;
    Ok(())
}
