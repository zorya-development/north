use leptos::prelude::*;
use north_domain::Task;

use crate::server_fns::projects::{clear_task_project, set_task_project};
use crate::server_fns::tags::set_task_tags;
use crate::server_fns::tasks::*;

#[derive(Clone)]
pub struct TaskStore {
    pub on_toggle_complete: Callback<(i64, bool)>,
    pub on_delete: Callback<i64>,
    pub on_update: Callback<(i64, String, Option<String>)>,
    pub on_set_start_at: Callback<(i64, String)>,
    pub on_clear_start_at: Callback<i64>,
    pub on_set_project: Callback<(i64, i64)>,
    pub on_clear_project: Callback<i64>,
    pub on_set_tags: Callback<(i64, Vec<String>)>,
    pub on_review: Callback<i64>,
    pub on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
}

impl TaskStore {
    pub fn new(resource: Resource<Result<Vec<Task>, ServerFnError>>) -> Self {
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

        let update_action = Action::new(|input: &(i64, String, Option<String>)| {
            let (id, title, body) = input.clone();
            update_task(id, title, body)
        });

        let set_start_at_action = Action::new(|input: &(i64, String)| {
            let (id, start_at) = input.clone();
            set_task_start_at(id, start_at)
        });

        let clear_start_at_action = Action::new(|id: &i64| {
            let id = *id;
            clear_task_start_at(id)
        });

        let set_project_action = Action::new(|input: &(i64, i64)| {
            let (task_id, project_id) = *input;
            set_task_project(task_id, project_id)
        });

        let clear_project_action = Action::new(|id: &i64| {
            let id = *id;
            clear_task_project(id)
        });

        let set_tags_action = Action::new(|input: &(i64, Vec<String>)| {
            let (task_id, tag_names) = input.clone();
            set_task_tags(task_id, tag_names)
        });

        let review_action = Action::new(|id: &i64| {
            let id = *id;
            review_task(id)
        });

        let reorder_action = Action::new(|input: &(i64, String, Option<Option<i64>>)| {
            let (task_id, sort_key, parent_id) = input.clone();
            let (change_parent, new_parent_id) = match parent_id {
                Some(pid) => (true, pid),
                None => (false, None),
            };
            reorder_task(task_id, sort_key, change_parent, new_parent_id)
        });

        // Complete/uncomplete: do NOT refetch — task stays in place so the
        // user can undo a misclick. TaskListView handles local list movement.

        Effect::new(move || {
            if let Some(Ok(_)) = delete_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = update_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = set_start_at_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = clear_start_at_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = set_project_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = clear_project_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = set_tags_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = review_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = reorder_action.value().get() {
                resource.refetch();
            }
        });

        Self {
            on_toggle_complete: Callback::new(move |(id, was_completed): (i64, bool)| {
                if was_completed {
                    uncomplete_action.dispatch(id);
                } else {
                    complete_action.dispatch(id);
                }
            }),
            on_delete: Callback::new(move |id: i64| {
                delete_action.dispatch(id);
            }),
            on_update: Callback::new(move |(id, title, body): (i64, String, Option<String>)| {
                update_action.dispatch((id, title, body));
            }),
            on_set_start_at: Callback::new(move |(id, start_at): (i64, String)| {
                set_start_at_action.dispatch((id, start_at));
            }),
            on_clear_start_at: Callback::new(move |id: i64| {
                clear_start_at_action.dispatch(id);
            }),
            on_set_project: Callback::new(move |(task_id, project_id): (i64, i64)| {
                set_project_action.dispatch((task_id, project_id));
            }),
            on_clear_project: Callback::new(move |id: i64| {
                clear_project_action.dispatch(id);
            }),
            on_set_tags: Callback::new(move |(task_id, tag_names): (i64, Vec<String>)| {
                set_tags_action.dispatch((task_id, tag_names));
            }),
            on_review: Callback::new(move |id: i64| {
                review_action.dispatch(id);
            }),
            on_reorder: Callback::new(
                move |(task_id, sort_key, parent_id): (i64, String, Option<Option<i64>>)| {
                    reorder_action.dispatch((task_id, sort_key, parent_id));
                },
            ),
        }
    }
}
