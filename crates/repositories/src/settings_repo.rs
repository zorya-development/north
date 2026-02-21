use leptos::prelude::ServerFnError;
use north_dto::{UpdateSettings, UserSettings};

use crate::notify_on_error;

pub struct SettingsRepository;

impl SettingsRepository {
    pub async fn get() -> Result<UserSettings, ServerFnError> {
        notify_on_error(north_server_fns::settings::get_user_settings().await)
    }

    pub async fn update(input: UpdateSettings) -> Result<(), ServerFnError> {
        notify_on_error(north_server_fns::settings::update_settings(input).await)
    }
}
