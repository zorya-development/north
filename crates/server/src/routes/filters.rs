use axum::extract::{Path, State};
use axum::Json;
use north_core::FilterService;
use north_dto::SavedFilter;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::AppState;

pub async fn list_filters(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<SavedFilter>>, AppError> {
    let results = FilterService::list(&state.pool, auth_user.id).await?;
    Ok(Json(results))
}

pub async fn delete_filter(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    FilterService::delete(&state.pool, auth_user.id, id).await?;
    Ok(())
}
