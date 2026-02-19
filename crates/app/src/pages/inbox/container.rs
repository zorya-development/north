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
            root_task_ids=ctrl.root_task_ids
            show_completed=ctrl.show_completed
            completed_count=ctrl.completed_count
            is_loaded=ctrl.is_loaded
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            on_reorder=Callback::new(move |(id, key, parent)| {
                ctrl.reorder_task(id, key, parent)
            })
        />
    }
}
