use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::AllTasksController;
use super::view::AllTasksView;

#[component]
pub fn AllTasksPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = AllTasksController::new(app_store);

    view! {
        <AllTasksView
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
