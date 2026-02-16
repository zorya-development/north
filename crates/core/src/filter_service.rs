use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::models::{NewSavedFilter, SavedFilterChangeset, SavedFilterRow};
use north_db::schema::saved_filters;
use north_db::DbPool;
use north_domain::SavedFilter;

use crate::{ServiceError, ServiceResult};

pub struct FilterService;

impl FilterService {
    pub async fn list(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<SavedFilter>> {
        let mut conn = pool.get().await?;
        let rows = saved_filters::table
            .filter(saved_filters::user_id.eq(user_id))
            .order(saved_filters::position.asc())
            .select(SavedFilterRow::as_select())
            .load(&mut conn)
            .await?;
        Ok(rows.into_iter().map(SavedFilter::from).collect())
    }

    pub async fn get_by_id(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<SavedFilter> {
        let mut conn = pool.get().await?;
        let row = saved_filters::table
            .filter(saved_filters::id.eq(id))
            .filter(saved_filters::user_id.eq(user_id))
            .select(SavedFilterRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Filter not found".into()))?;
        Ok(SavedFilter::from(row))
    }

    pub async fn create(
        pool: &DbPool,
        user_id: i64,
        title: &str,
        query: &str,
    ) -> ServiceResult<SavedFilter> {
        north_domain::parse_filter(query).map_err(|errs| {
            ServiceError::BadRequest(
                errs.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;

        let mut conn = pool.get().await?;

        let max_pos: Option<i32> = saved_filters::table
            .filter(saved_filters::user_id.eq(user_id))
            .select(diesel::dsl::max(saved_filters::position))
            .first(&mut conn)
            .await?;
        let position = max_pos.unwrap_or(0) + 1;

        let row = diesel::insert_into(saved_filters::table)
            .values(&NewSavedFilter {
                user_id,
                title,
                query,
                position,
            })
            .returning(SavedFilterRow::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(SavedFilter::from(row))
    }

    pub async fn update(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        title: Option<&str>,
        query: Option<&str>,
        position: Option<i32>,
    ) -> ServiceResult<SavedFilter> {
        if let Some(q) = query {
            north_domain::parse_filter(q).map_err(|errs| {
                ServiceError::BadRequest(
                    errs.into_iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; "),
                )
            })?;
        }

        let mut conn = pool.get().await?;
        let row = diesel::update(
            saved_filters::table
                .filter(saved_filters::id.eq(id))
                .filter(saved_filters::user_id.eq(user_id)),
        )
        .set(&SavedFilterChangeset {
            title,
            query,
            position,
        })
        .returning(SavedFilterRow::as_returning())
        .get_result(&mut conn)
        .await
        .optional()?
        .ok_or_else(|| ServiceError::NotFound("Filter not found".into()))?;

        Ok(SavedFilter::from(row))
    }

    pub async fn delete(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::delete(
            saved_filters::table
                .filter(saved_filters::id.eq(id))
                .filter(saved_filters::user_id.eq(user_id)),
        )
        .execute(&mut conn)
        .await?;

        if affected == 0 {
            return Err(ServiceError::NotFound("Filter not found".into()));
        }

        Ok(())
    }
}
