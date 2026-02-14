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

    let on_toggle = Callback::new(move |(id, done)| {
        ctrl.toggle_complete(id, done);
    });
    let on_delete = Callback::new(move |id| ctrl.delete_task(id));
    let on_update = Callback::new(move |(id, title, body): (i64, String, Option<String>)| {
        ctrl.update_task(id, title, body);
    });
    let on_create = Callback::new(move |(title, body): (String, Option<String>)| {
        ctrl.create_task(title, body);
    });
    let on_set_start_at = Callback::new(move |(id, start_at): (i64, String)| {
        ctrl.set_start_at(id, start_at);
    });
    let on_clear_start_at = Callback::new(move |id: i64| ctrl.clear_start_at(id));
    let on_set_project = Callback::new(move |(task_id, project_id): (i64, i64)| {
        ctrl.set_project(task_id, project_id);
    });
    let on_clear_project = Callback::new(move |id: i64| ctrl.clear_project(id));
    let on_set_tags = Callback::new(move |(task_id, tags): (i64, Vec<String>)| {
        ctrl.set_tags(task_id, tags);
    });
    let on_review = Callback::new(move |id: i64| ctrl.review_task(id));
    let on_reorder = Callback::new(
        move |(task_id, sort_key, parent_id): (i64, String, Option<Option<i64>>)| {
            ctrl.reorder_task(task_id, sort_key, parent_id);
        },
    );

    view! {
        <InboxView
            tasks=ctrl.tasks
            on_toggle_complete=on_toggle
            on_delete=on_delete
            on_update=on_update
            on_create=on_create
            on_set_start_at=on_set_start_at
            on_clear_start_at=on_clear_start_at
            on_set_project=on_set_project
            on_clear_project=on_clear_project
            on_set_tags=on_set_tags
            on_review=on_review
            on_reorder=on_reorder
        />
    }
}
