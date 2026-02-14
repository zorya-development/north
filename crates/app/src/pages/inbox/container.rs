use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::InboxController;
use super::view::InboxView;
use crate::components::drag_drop::DragDropContext;

#[component]
pub fn InboxPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = InboxController::new(app_store);

    provide_context(DragDropContext::new());

    view! {
        <InboxView
            tasks=ctrl.tasks
            on_toggle_complete=Callback::new(move |(id, done)| ctrl.toggle_complete(id, done))
            on_delete=Callback::new(move |id| ctrl.delete_task(id))
            on_update=Callback::new(move |(id, title, body): (i64, String, Option<String>)| ctrl.update_task(id, title, body))
            on_create=Callback::new(move |(title, body): (String, Option<String>)| ctrl.create_task(title, body))
            on_set_start_at=Callback::new(move |(id, start_at): (i64, String)| ctrl.set_start_at(id, start_at))
            on_clear_start_at=Callback::new(move |id: i64| ctrl.clear_start_at(id))
            on_set_project=Callback::new(move |(task_id, project_id): (i64, i64)| ctrl.set_project(task_id, project_id))
            on_clear_project=Callback::new(move |id: i64| ctrl.clear_project(id))
            on_set_tags=Callback::new(move |(task_id, tags): (i64, Vec<String>)| ctrl.set_tags(task_id, tags))
            on_review=Callback::new(move |id: i64| ctrl.review_task(id))
            on_reorder=Callback::new(move |(task_id, sort_key, parent_id): (i64, String, Option<Option<i64>>)| ctrl.reorder_task(task_id, sort_key, parent_id))
        />
    }
}
