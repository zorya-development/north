use axum::extract::State;
use axum::Json;
use north_core::stats_service::Stats;
use north_core::StatsService;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::AppState;

pub async fn get_stats(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<Stats>, AppError> {
    let stats = StatsService::get_stats(&state.pool, auth_user.id).await?;
    Ok(Json(stats))
}
