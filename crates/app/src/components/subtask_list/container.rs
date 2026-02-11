use leptos::prelude::*;
use north_domain::TaskWithMeta;

use super::view::SubtaskListView;
use crate::server_fns::tasks::*;

#[component]
pub fn SubtaskList(
    parent_id: i64,
    parent_sequential_limit: i16,
    #[prop(default = 0)] depth: usize,
    project_id: Option<i64>,
    on_navigate_to: Callback<i64>,
    on_parent_refetch: Callback<()>,
) -> impl IntoView {
    let subtasks: Resource<Result<Vec<TaskWithMeta>, ServerFnError>> =
        Resource::new(move || parent_id, |pid| get_subtasks(pid));

    let (hide_completed, set_hide_completed) = signal(false);

    let complete_action = Action::new(|id: &i64| {
        let id = *id;
        complete_task(id)
    });

    let uncomplete_action = Action::new(|id: &i64| {
        let id = *id;
        uncomplete_task(id)
    });

    let delete_action = Action::new(|id: &i64| {
        let id = *id;
        delete_task(id)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = complete_action.value().get() {
            subtasks.refetch();
            on_parent_refetch.run(());
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = uncomplete_action.value().get() {
            subtasks.refetch();
            on_parent_refetch.run(());
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = delete_action.value().get() {
            subtasks.refetch();
            on_parent_refetch.run(());
        }
    });

    let on_toggle_complete = Callback::new(move |(id, was_completed): (i64, bool)| {
        if was_completed {
            uncomplete_action.dispatch(id);
        } else {
            complete_action.dispatch(id);
        }
    });

    let on_delete = Callback::new(move |id: i64| {
        delete_action.dispatch(id);
    });

    let on_created = Callback::new(move |()| {
        subtasks.refetch();
        on_parent_refetch.run(());
    });

    view! {
        <SubtaskListView
            subtasks=subtasks
            parent_id=parent_id
            depth=depth
            project_id=project_id
            hide_completed=hide_completed
            set_hide_completed=set_hide_completed
            on_toggle_complete=on_toggle_complete
            on_delete=on_delete
            on_navigate_to=on_navigate_to
            on_parent_refetch=on_parent_refetch
            on_created=on_created
            parent_sequential_limit=parent_sequential_limit
        />
    }
}
