use leptos::prelude::*;

use crate::components::drag_drop::DragDropContext;
use crate::components::task_detail_modal::{TaskDetailContext, TaskDetailModal};
use crate::components::task_form::InlineTaskForm;
use crate::components::task_list::TaskList;
use crate::server_fns::tasks::{create_task, get_completed_tasks, get_inbox_tasks};
use crate::stores::task_store::TaskStore;

#[component]
pub fn InboxPage() -> impl IntoView {
    let open_task_id = RwSignal::new(None::<i64>);
    provide_context(TaskDetailContext { open_task_id });
    provide_context(DragDropContext::new());

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

    let task_ids = Signal::derive(move || {
        inbox_tasks
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .iter()
            .map(|t| t.task.id)
            .collect::<Vec<_>>()
    });

    view! {
        <div class="space-y-4">
            <h1 class="text-2xl font-semibold tracking-tight text-text-primary">"Inbox"</h1>
            <InlineTaskForm on_submit=on_create/>
            <TaskList
                resource=inbox_tasks
                store=store.clone()
                empty_message="No tasks in your inbox. Add one above."
                completed_resource=completed
                draggable=true
            />
            <TaskDetailModal task_ids=task_ids task_store=store/>
        </div>
    }
}
