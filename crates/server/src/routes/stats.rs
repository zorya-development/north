use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct Stats {
    pub created_today: i64,
    pub completed_today: i64,
    pub created_week: i64,
    pub completed_week: i64,
    pub total_open: i64,
    pub total_completed: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct StatsRow {
    created_today: i64,
    completed_today: i64,
    created_week: i64,
    completed_week: i64,
    total_open: i64,
    total_completed: i64,
}

pub async fn get_stats(
    auth_user: axum::Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<Stats>, AppError> {
    let row = sqlx::query_as::<_, StatsRow>(
        "SELECT \
            COUNT(*) FILTER ( \
                WHERE created_at::date = CURRENT_DATE \
            ) as created_today, \
            COUNT(*) FILTER ( \
                WHERE completed_at::date = CURRENT_DATE \
            ) as completed_today, \
            COUNT(*) FILTER ( \
                WHERE created_at >= date_trunc('week', CURRENT_DATE) \
            ) as created_week, \
            COUNT(*) FILTER ( \
                WHERE completed_at >= date_trunc('week', CURRENT_DATE) \
            ) as completed_week, \
            COUNT(*) FILTER ( \
                WHERE completed_at IS NULL \
            ) as total_open, \
            COUNT(*) FILTER ( \
                WHERE completed_at IS NOT NULL \
            ) as total_completed \
         FROM tasks \
         WHERE user_id = $1",
    )
    .bind(auth_user.id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(Stats {
        created_today: row.created_today,
        completed_today: row.completed_today,
        created_week: row.created_week,
        completed_week: row.completed_week,
        total_open: row.total_open,
        total_completed: row.total_completed,
    }))
}
