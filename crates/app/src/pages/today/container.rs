use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::TodayController;
use super::view::TodayView;

#[component]
pub fn TodayPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = TodayController::new(app_store);

    view! {
        <TodayView
            view=ctrl.view
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            toolbar=ctrl.toolbar_config()
            show_keybindings_help=RwSignal::new(false)
        />
    }
}
