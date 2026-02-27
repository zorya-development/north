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
            hide_non_actionable=ctrl.hide_non_actionable
            node_filter=ctrl.node_filter
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            on_reorder=Callback::new(move |(id, key, parent)| {
                ctrl.reorder_task(id, key, parent)
            })
            on_toggle_visibility=Callback::new(move |()| {
                ctrl.toggle_actionable_visibility()
            })
        />
    }
}
