use leptos::prelude::ServerFnError;
use north_dto::UserSettings;

pub struct SettingsRepository;

impl SettingsRepository {
    pub async fn get() -> Result<UserSettings, ServerFnError> {
        north_server_fns::settings::get_user_settings().await
    }

    pub async fn update_review_interval(days: i16) -> Result<(), ServerFnError> {
        north_server_fns::settings::update_review_interval(days).await
    }
}
