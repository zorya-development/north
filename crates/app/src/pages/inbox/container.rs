use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::InboxController;
use super::view::InboxView;

#[component]
pub fn InboxPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = InboxController::new(app_store);

    view! {
        <InboxView
            active_task_ids=ctrl.active_task_ids
            completed_task_ids=ctrl.completed_task_ids
            completed_count=ctrl.completed_count
            is_loaded=ctrl.is_loaded
            on_add_task=Callback::new(move |()| ctrl.open_create())
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            on_reorder=Callback::new(move |(id, key, parent)| {
                ctrl.reorder_task(id, key, parent)
            })
            active_tasks_for_reorder=ctrl.active_tasks_for_reorder
        />
    }
}
