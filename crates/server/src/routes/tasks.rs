use axum::extract::{Path, Query, State};
use axum::Json;
use north_core::TaskService;
use north_dto::{CreateTask, Task, TaskFilter, UpdateTask};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::AppState;

pub async fn list_tasks(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Query(filter): Query<TaskFilter>,
) -> Result<Json<Vec<Task>>, AppError> {
    let results = TaskService::list(&state.pool, auth_user.id, &filter).await?;
    Ok(Json(results))
}

pub async fn create_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Json(body): Json<CreateTask>,
) -> Result<Json<Task>, AppError> {
    let task = TaskService::create(&state.pool, auth_user.id, &body).await?;
    Ok(Json(task))
}

pub async fn get_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Task>, AppError> {
    let task = TaskService::get_by_id(&state.pool, auth_user.id, id).await?;
    Ok(Json(task))
}

pub async fn update_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTask>,
) -> Result<Json<Task>, AppError> {
    let task = TaskService::update(&state.pool, auth_user.id, id, &body).await?;
    Ok(Json(task))
}

pub async fn delete_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    TaskService::delete(&state.pool, auth_user.id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn review_task(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Task>, AppError> {
    let task = TaskService::review(&state.pool, auth_user.id, id).await?;
    Ok(Json(task))
}
