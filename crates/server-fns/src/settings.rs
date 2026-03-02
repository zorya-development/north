use leptos::prelude::*;
use north_dto::{UpdateSettings, UserSettings};

#[server(ApiGetUserSettingsFn, "/api")]
pub async fn get_user_settings() -> Result<UserSettings, ServerFnError> {
    with_auth!(|pool, uid| north_core::UserService::get_settings(pool, uid))
}

#[server(ApiUpdateSettingsFn, "/api")]
pub async fn update_settings(input: UpdateSettings) -> Result<(), ServerFnError> {
    if let Some(days) = input.review_interval_days {
        if days < 1 {
            return Err(ServerFnError::new(
                "Review interval must be at least 1 day".to_string(),
            ));
        }
    }

    with_auth!(|pool, uid| north_core::UserService::update_settings(pool, uid, &input))
}
