use leptos::prelude::*;
use north_recurrence::RecurrenceType;
#[cfg(feature = "hydrate")]
use north_repositories::SettingsRepository;

use crate::libs::ReactiveRecurrenceRule;

pub fn build_reactive_rule(
    existing_type: Option<RecurrenceType>,
    existing_rule: Option<String>,
) -> ReactiveRecurrenceRule {
    let tz = RwSignal::new(String::from("UTC"));
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            leptos::task::spawn_local(async move {
                if let Ok(settings) = SettingsRepository::get().await {
                    tz.set(settings.timezone);
                }
            });
        });
    }

    ReactiveRecurrenceRule::from_str(
        existing_type,
        existing_rule,
        Signal::derive(move || tz.get()),
    )
}
