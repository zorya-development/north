use leptos::prelude::*;
use north_domain::UserSettings;

#[server(GetUserSettingsFn, "/api")]
pub async fn get_user_settings() -> Result<UserSettings, ServerFnError> {
    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::UserService::get_settings(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(UpdateReviewIntervalFn, "/api")]
pub async fn update_review_interval(days: i16) -> Result<(), ServerFnError> {
    if days < 1 {
        return Err(ServerFnError::new(
            "Review interval must be at least 1 day".to_string(),
        ));
    }

    let pool = expect_context::<north_services::DbPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;
    north_services::UserService::update_review_interval(&pool, user_id, days)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
