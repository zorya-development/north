use leptos::prelude::*;

use crate::components::task_form::InlineTaskForm;
use crate::components::task_list::TaskList;
use crate::server_fns::tasks::{create_task, get_completed_tasks, get_inbox_tasks};
use crate::stores::task_store::TaskStore;

#[component]
pub fn InboxPage() -> impl IntoView {
    let inbox_tasks = Resource::new(|| (), |_| get_inbox_tasks());
    let completed = Resource::new(|| (), |_| get_completed_tasks(None, true));
    let store = TaskStore::new(inbox_tasks);

    let create_action = Action::new(|input: &(String, Option<String>)| {
        let (title, body) = input.clone();
        create_task(title, body)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = create_action.value().get() {
            inbox_tasks.refetch();
        }
    });

    let on_create = move |title: String, body: Option<String>| {
        create_action.dispatch((title, body));
    };

    view! {
        <div class="space-y-4">
            <h1 class="text-xl font-semibold text-text-primary">"Inbox"</h1>
            <InlineTaskForm on_submit=on_create/>
            <TaskList
                resource=inbox_tasks
                store=store
                empty_message="No tasks in your inbox. Add one above."
                completed_resource=completed
            />
        </div>
    }
}
