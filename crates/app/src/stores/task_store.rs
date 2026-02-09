use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::server_fns::tasks::*;

#[derive(Clone)]
pub struct TaskStore {
    pub on_toggle_complete: Callback<(i64, bool)>,
    pub on_delete: Callback<i64>,
    pub on_update: Callback<(i64, String, Option<String>)>,
    pub on_set_start_at: Callback<(i64, String)>,
    pub on_clear_start_at: Callback<i64>,
}

impl TaskStore {
    pub fn new(resource: Resource<Result<Vec<TaskWithMeta>, ServerFnError>>) -> Self {
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

        Effect::new(move || {
            if let Some(Ok(_)) = complete_action.value().get() {
                resource.refetch();
            }
        });

        Effect::new(move || {
            if let Some(Ok(_)) = uncomplete_action.value().get() {
                resource.refetch();
            }
        });

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
        }
    }
}
