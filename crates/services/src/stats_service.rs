use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::DbPool;
use serde::Serialize;

use crate::ServiceResult;

#[derive(Debug, Serialize)]
pub struct Stats {
    pub created_today: i64,
    pub completed_today: i64,
    pub created_week: i64,
    pub completed_week: i64,
    pub total_open: i64,
    pub total_completed: i64,
}

pub struct StatsService;

impl StatsService {
    pub async fn get_stats(pool: &DbPool, user_id: i64) -> ServiceResult<Stats> {
        let mut conn = pool.get().await?;

        // Use raw SQL for COUNT FILTER which Diesel doesn't natively support
        let row: (i64, i64, i64, i64, i64, i64) = diesel::sql_query(
            "SELECT \
                COUNT(*) FILTER (WHERE created_at::date = CURRENT_DATE) as created_today, \
                COUNT(*) FILTER (WHERE completed_at::date = CURRENT_DATE) as completed_today, \
                COUNT(*) FILTER (WHERE created_at >= date_trunc('week', CURRENT_DATE)) as created_week, \
                COUNT(*) FILTER (WHERE completed_at >= date_trunc('week', CURRENT_DATE)) as completed_week, \
                COUNT(*) FILTER (WHERE completed_at IS NULL) as total_open, \
                COUNT(*) FILTER (WHERE completed_at IS NOT NULL) as total_completed \
             FROM tasks WHERE user_id = $1",
        )
        .bind::<diesel::sql_types::Int8, _>(user_id)
        .get_result::<StatsRow>(&mut conn)
        .await
        .map(|r| (
            r.created_today,
            r.completed_today,
            r.created_week,
            r.completed_week,
            r.total_open,
            r.total_completed,
        ))?;

        Ok(Stats {
            created_today: row.0,
            completed_today: row.1,
            created_week: row.2,
            completed_week: row.3,
            total_open: row.4,
            total_completed: row.5,
        })
    }
}

#[derive(Debug, QueryableByName)]
struct StatsRow {
    #[diesel(sql_type = diesel::sql_types::Int8)]
    created_today: i64,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    completed_today: i64,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    created_week: i64,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    completed_week: i64,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    total_open: i64,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    total_completed: i64,
}
