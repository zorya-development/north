use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::UpdateSettings;
use north_repositories::SettingsRepository;

#[derive(Clone, Copy)]
pub struct SettingsController {
    pub interval: (ReadSignal<String>, WriteSignal<String>),
    pub saved: (ReadSignal<bool>, WriteSignal<bool>),
    pub is_loaded: Signal<bool>,
}

impl SettingsController {
    pub fn new() -> Self {
        let interval = signal(String::new());
        let saved = signal(false);
        let loaded = RwSignal::new(false);

        let set_interval = interval.1;
        spawn_local(async move {
            if let Ok(settings) = SettingsRepository::get().await {
                set_interval.set(settings.review_interval_days.to_string());
            }
            loaded.set(true);
        });

        let is_loaded = Signal::derive(move || loaded.get());

        Self {
            interval,
            saved,
            is_loaded,
        }
    }

    pub fn save(&self) {
        let interval_str = self.interval.0.get_untracked();
        let set_saved = self.saved.1;

        if let Ok(days) = interval_str.parse::<i16>() {
            if days >= 1 {
                spawn_local(async move {
                    let input = UpdateSettings {
                        review_interval_days: Some(days),
                        ..Default::default()
                    };
                    if SettingsRepository::update(input).await.is_ok() {
                        set_saved.set(true);
                    }
                });
            }
        }
    }
}
