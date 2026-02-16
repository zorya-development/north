use leptos::prelude::*;
use north_dto::{UpdateSettings, UserSettings};

#[server(ApiGetUserSettingsFn, "/api")]
pub async fn get_user_settings() -> Result<UserSettings, ServerFnError> {
    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::UserService::get_settings(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(ApiUpdateReviewIntervalFn, "/api")]
pub async fn update_review_interval(days: i16) -> Result<(), ServerFnError> {
    if days < 1 {
        return Err(ServerFnError::new(
            "Review interval must be at least 1 day".to_string(),
        ));
    }

    let pool = expect_context::<north_core::DbPool>();
    let user_id = crate::auth::get_auth_user_id().await?;
    north_core::UserService::update_settings(
        &pool,
        user_id,
        &UpdateSettings {
            review_interval_days: Some(days),
            ..Default::default()
        },
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}
