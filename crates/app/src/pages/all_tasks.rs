use leptos::prelude::*;

use crate::components::task_detail_modal::{TaskDetailContext, TaskDetailModal};
use crate::components::task_list::TaskList;
use crate::server_fns::tasks::{get_all_tasks, get_completed_tasks};
use crate::stores::task_store::TaskStore;

#[component]
pub fn AllTasksPage() -> impl IntoView {
    let open_task_id = RwSignal::new(None::<i64>);
    provide_context(TaskDetailContext { open_task_id });

    let all_tasks = Resource::new(|| (), |_| get_all_tasks());
    let completed = Resource::new(|| (), |_| get_completed_tasks(None, false));
    let store = TaskStore::new(all_tasks);

    let task_ids = Signal::derive(move || {
        all_tasks
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .iter()
            .map(|t| t.task.id)
            .collect::<Vec<_>>()
    });

    view! {
        <div class="space-y-4">
            <h1 class="text-2xl font-semibold tracking-tight text-text-primary">
                "All Tasks"
            </h1>
            <TaskList
                resource=all_tasks
                store=store.clone()
                empty_message="No tasks yet."
                completed_resource=completed
            />
            <TaskDetailModal task_ids=task_ids task_store=store/>
        </div>
    }
}
