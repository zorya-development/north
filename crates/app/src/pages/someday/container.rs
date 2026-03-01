use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::SomedayController;
use super::view::SomedayView;

#[component]
pub fn SomedayPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = SomedayController::new(app_store);

    view! {
        <SomedayView
            root_task_ids=ctrl.root_task_ids
            is_loaded=ctrl.is_loaded
            node_filter=ctrl.node_filter
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            on_reorder=Callback::new(move |(id, key, parent)| {
                ctrl.reorder_task(id, key, parent)
            })
            toolbar=ctrl.toolbar_config()
            show_keybindings_help=RwSignal::new(false)
        />
    }
}
