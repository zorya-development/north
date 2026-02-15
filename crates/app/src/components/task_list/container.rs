use leptos::prelude::*;
use north_domain::Task;

use super::view::TaskListView;
use crate::stores::task_store::TaskStore;

#[component]
pub fn TaskList(
    resource: Resource<Result<Vec<Task>, ServerFnError>>,
    store: TaskStore,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    #[prop(optional)] completed_resource: Option<
        Resource<Result<Vec<Task>, ServerFnError>>,
    >,
    #[prop(default = false)] draggable: bool,
) -> impl IntoView {
    match completed_resource {
        Some(cr) => view! {
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
                on_reorder=store.on_reorder
                show_review=show_review
                show_project=show_project
                empty_message=empty_message
                completed_resource=cr
                draggable=draggable
            />
        }
        .into_any(),
        None => view! {
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
                on_reorder=store.on_reorder
                show_review=show_review
                show_project=show_project
                empty_message=empty_message
                draggable=draggable
            />
        }
        .into_any(),
    }
}
