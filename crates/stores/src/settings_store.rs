use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::{UpdateSettings, UserSettings};
use north_repositories::SettingsRepository;

#[derive(Clone, Copy)]
pub struct SettingsStore {
    settings: RwSignal<UserSettings>,
    loaded: RwSignal<bool>,
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsStore {
    pub fn new() -> Self {
        Self {
            settings: RwSignal::new(UserSettings::default()),
            loaded: RwSignal::new(false),
        }
    }

    pub fn refetch(&self) {
        let store = *self;
        spawn_local(async move {
            if let Ok(settings) = SettingsRepository::get().await {
                store.settings.set(settings);
                store.loaded.set(true);
            }
        });
    }

    pub fn get(&self) -> UserSettings {
        self.settings.get()
    }

    pub fn timezone(&self) -> Signal<String> {
        let settings = self.settings;
        Signal::derive(move || settings.get().timezone)
    }

    pub fn review_interval_days(&self) -> Signal<i64> {
        let settings = self.settings;
        Signal::derive(move || settings.get().review_interval_days as i64)
    }

    pub fn update(&self, input: UpdateSettings) {
        let store = *self;
        spawn_local(async move {
            if SettingsRepository::update(input).await.is_ok() {
                store.refetch_async().await;
            }
        });
    }

    pub async fn update_async(&self, input: UpdateSettings) -> bool {
        if SettingsRepository::update(input).await.is_ok() {
            self.refetch_async().await;
            true
        } else {
            false
        }
    }

    async fn refetch_async(&self) {
        if let Ok(settings) = SettingsRepository::get().await {
            self.settings.set(settings);
            self.loaded.set(true);
        }
    }
}
