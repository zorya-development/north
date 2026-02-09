use leptos::prelude::*;
use north_domain::UserSettings;

#[server(GetUserSettingsFn, "/api")]
pub async fn get_user_settings() -> Result<UserSettings, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let settings: serde_json::Value = sqlx::query_scalar(
        "SELECT settings FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_settings: UserSettings =
        serde_json::from_value(settings).unwrap_or_default();

    Ok(user_settings)
}

#[server(UpdateReviewIntervalFn, "/api")]
pub async fn update_review_interval(days: i16) -> Result<(), ServerFnError> {
    if days < 1 {
        return Err(ServerFnError::new(
            "Review interval must be at least 1 day".to_string(),
        ));
    }

    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE users SET settings = jsonb_set(\
             settings, '{review_interval_days}', to_jsonb($1::smallint)\
         ) WHERE id = $2",
    )
    .bind(days)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("User not found".to_string()));
    }

    Ok(())
}
