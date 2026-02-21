use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::models::UserRow;
use north_db::schema::users;
use north_db::DbPool;
use north_dto::{UpdateSettings, UserSettings};

use crate::{ServiceError, ServiceResult};

pub struct UserService;

impl UserService {
    pub async fn get_by_email(pool: &DbPool, email: &str) -> ServiceResult<Option<UserRow>> {
        let mut conn = pool.get().await?;
        let row = users::table
            .filter(users::email.eq(email))
            .select(UserRow::as_select())
            .first(&mut conn)
            .await
            .optional()?;
        Ok(row)
    }

    pub async fn get_settings(pool: &DbPool, user_id: i64) -> ServiceResult<UserSettings> {
        let mut conn = pool.get().await?;
        let val: serde_json::Value = users::table
            .filter(users::id.eq(user_id))
            .select(users::settings)
            .first(&mut conn)
            .await?;
        Ok(serde_json::from_value(val).unwrap_or_default())
    }

    pub async fn update_settings(
        pool: &DbPool,
        user_id: i64,
        input: &UpdateSettings,
    ) -> ServiceResult<()> {
        let mut settings = Self::get_settings(pool, user_id).await?;

        if let Some(days) = input.review_interval_days {
            settings.review_interval_days = days;
        }
        if let Some(ref tz) = input.timezone {
            settings.timezone = tz.clone();
        }
        if let Some(collapsed) = input.sidebar_collapsed {
            settings.sidebar_collapsed = collapsed;
        }

        let val =
            serde_json::to_value(&settings).map_err(|e| ServiceError::BadRequest(e.to_string()))?;

        let mut conn = pool.get().await?;
        let affected = diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::settings.eq(val))
            .execute(&mut conn)
            .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("User not found".into()));
        }
        Ok(())
    }

    pub async fn admin_exists(pool: &DbPool) -> ServiceResult<bool> {
        use north_db::sql_types::UserRoleMapping;

        let mut conn = pool.get().await?;
        let count: i64 = users::table
            .filter(users::role.eq(UserRoleMapping::Admin))
            .count()
            .get_result(&mut conn)
            .await?;
        Ok(count > 0)
    }

    pub async fn create_admin(
        pool: &DbPool,
        email: &str,
        password_hash: &str,
        name: &str,
        settings: serde_json::Value,
    ) -> ServiceResult<()> {
        use north_db::models::NewUser;
        use north_db::sql_types::UserRoleMapping;

        let mut conn = pool.get().await?;
        diesel::insert_into(users::table)
            .values(&NewUser {
                email,
                password_hash,
                name,
                role: UserRoleMapping::Admin,
                settings,
            })
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}
