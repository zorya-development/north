use std::sync::Arc;

use leptos::prelude::*;
use north_domain::TaskWithMeta;

use super::view::TaskCardView;

#[component]
pub fn TaskCard(
    task: TaskWithMeta,
    on_toggle_complete: Callback<(i64, bool)>,
    on_delete: Callback<i64>,
    on_update: Callback<(i64, String, Option<String>)>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    on_set_tags: Callback<(i64, Vec<String>)>,
    on_review: Callback<i64>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = false)] draggable: bool,
    #[prop(default = 0)] depth: u8,
) -> impl IntoView {
    let task_id = task.task.id;
    let title = task.task.title.clone();
    let body = task.task.body.clone();
    let sort_key = task.task.sort_key.clone();
    let parent_id = task.task.parent_id;
    let project_id = task.task.project_id;
    let project_title = task.project_title.clone();
    let due_date = task.task.due_date;
    let start_at = task.task.start_at;
    let reviewed_at = task.task.reviewed_at;
    let initial_completed = task.task.completed_at.is_some();
    let tags = task.tags.clone();
    let subtask_count = task.subtask_count;
    let completed_subtask_count = task.completed_subtask_count;

    let (is_completed, set_is_completed) = signal(initial_completed);
    let (editing, set_editing) = signal(false);
    let (menu_open, set_menu_open) = signal(false);

    let on_toggle = Callback::new(move |()| {
        let was_completed = is_completed.get_untracked();
        set_is_completed.set(!was_completed);
        on_toggle_complete.run((task_id, was_completed));
    });

    let on_delete_handler = Arc::new(move || {
        on_delete.run(task_id);
    });

    let on_save = Arc::new(move |new_title: String, new_body: Option<String>| {
        set_editing.set(false);
        on_update.run((task_id, new_title, new_body));
    });

    view! {
        <TaskCardView
            task_id=task_id
            title=title
            body=body
            sort_key=sort_key
            parent_id=parent_id
            project_id=project_id
            project_title=project_title
            due_date=due_date
            start_at=start_at
            reviewed_at=reviewed_at
            tags=tags
            is_completed=is_completed
            editing=editing
            set_editing=set_editing
            menu_open=menu_open
            set_menu_open=set_menu_open
            on_toggle=on_toggle
            on_delete=on_delete_handler
            on_save=on_save
            on_set_start_at=on_set_start_at
            on_clear_start_at=on_clear_start_at
            on_set_project=on_set_project
            on_clear_project=on_clear_project
            on_set_tags=on_set_tags
            on_review=on_review
            show_review=show_review
            show_project=show_project
            subtask_count=subtask_count
            completed_subtask_count=completed_subtask_count
            draggable=draggable
            depth=depth
        />
    }
}
