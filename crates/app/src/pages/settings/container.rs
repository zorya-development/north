use leptos::prelude::*;

use super::controller::SettingsController;
use super::view::SettingsView;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let ctrl = SettingsController::new();

    view! {
        <SettingsView
            interval=ctrl.interval.0
            set_interval=ctrl.interval.1
            timezone=ctrl.timezone.0
            set_timezone=ctrl.timezone.1
            is_loaded=ctrl.is_loaded
            on_save=Callback::new(move |()| ctrl.save())
        />
    }
}
