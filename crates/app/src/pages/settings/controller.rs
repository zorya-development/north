use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::UpdateSettings;
use north_stores::status_bar_store::StatusBarVariant;
use north_stores::use_app_store;

#[derive(Clone, Copy)]
pub struct SettingsController {
    pub interval: (ReadSignal<String>, WriteSignal<String>),
    pub timezone: (ReadSignal<String>, WriteSignal<String>),
    pub is_loaded: Signal<bool>,
}

impl SettingsController {
    pub fn new() -> Self {
        let app_store = use_app_store();
        let current = app_store.settings.get();

        let interval = signal(current.review_interval_days.to_string());
        let timezone = signal(current.timezone);
        let is_loaded = Signal::derive(move || true);

        Self {
            interval,
            timezone,
            is_loaded,
        }
    }

    pub fn save(&self) {
        let interval_str = self.interval.0.get_untracked();
        let tz = self.timezone.0.get_untracked();
        let app_store = use_app_store();

        if let Ok(days) = interval_str.parse::<i16>() {
            if days >= 1 {
                spawn_local(async move {
                    let input = UpdateSettings {
                        review_interval_days: Some(days),
                        timezone: Some(tz),
                        ..Default::default()
                    };
                    if app_store.settings.update_async(input).await {
                        app_store
                            .status_bar
                            .notify(StatusBarVariant::Success, "Settings saved");
                    }
                });
            }
        }
    }
}
