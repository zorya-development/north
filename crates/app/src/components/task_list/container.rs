use leptos::prelude::*;
use north_domain::Task;

use super::view::TaskListView;
use crate::components::drag_drop::DragDropContext;

#[component]
pub fn TaskList(
    active_task_ids: Memo<Vec<i64>>,
    active_tasks_for_reorder: Memo<Vec<Task>>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    #[prop(default = false)] compact: bool,
    #[prop(default = false)] draggable: bool,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    #[prop(optional)] on_task_click: Option<Callback<i64>>,
    is_loaded: Signal<bool>,
) -> impl IntoView {
    if !compact {
        provide_context(DragDropContext::new());
    }

    view! {
        <TaskListView
            active_task_ids=active_task_ids
            active_tasks_for_reorder=active_tasks_for_reorder
            show_review=show_review
            show_project=show_project
            empty_message=empty_message
            compact=compact
            draggable=draggable
            on_reorder=on_reorder
            on_task_click=on_task_click.unwrap_or(Callback::new(|_| {}))
            is_loaded=is_loaded
        />
    }
}
