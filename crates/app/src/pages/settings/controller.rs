use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::UpdateSettings;
use north_repositories::SettingsRepository;
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
        let interval = signal(String::new());
        let timezone = signal("UTC".to_string());
        let loaded = RwSignal::new(false);

        let set_interval = interval.1;
        let set_timezone = timezone.1;
        // Effect runs client-only, avoiding spawn_local panic during SSR.
        Effect::new(move || {
            spawn_local(async move {
                if let Ok(settings) = SettingsRepository::get().await {
                    set_interval.set(settings.review_interval_days.to_string());
                    set_timezone.set(settings.timezone);
                }
                loaded.set(true);
            });
        });

        let is_loaded = Signal::derive(move || loaded.get());

        Self {
            interval,
            timezone,
            is_loaded,
        }
    }

    pub fn save(&self) {
        let interval_str = self.interval.0.get_untracked();
        let tz = self.timezone.0.get_untracked();
        let status_bar = use_app_store().status_bar;

        if let Ok(days) = interval_str.parse::<i16>() {
            if days >= 1 {
                spawn_local(async move {
                    let input = UpdateSettings {
                        review_interval_days: Some(days),
                        timezone: Some(tz),
                        ..Default::default()
                    };
                    if SettingsRepository::update(input).await.is_ok() {
                        status_bar.notify(StatusBarVariant::Success, "Settings saved");
                    }
                });
            }
        }
    }
}
