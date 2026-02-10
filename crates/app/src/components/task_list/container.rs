use leptos::prelude::*;
use north_domain::TaskWithMeta;

use super::view::TaskListView;
use crate::stores::task_store::TaskStore;

#[component]
pub fn TaskList(
    resource: Resource<Result<Vec<TaskWithMeta>, ServerFnError>>,
    store: TaskStore,
    #[prop(default = false)] show_review: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
) -> impl IntoView {
    view! {
        <TaskListView
            resource=resource
            on_toggle_complete=store.on_toggle_complete
            on_delete=store.on_delete
            on_update=store.on_update
            on_set_start_at=store.on_set_start_at
            on_clear_start_at=store.on_clear_start_at
            on_set_project=store.on_set_project
            on_clear_project=store.on_clear_project
            on_set_tags=store.on_set_tags
            on_review=store.on_review
            show_review=show_review
            empty_message=empty_message
        />
    }
}
