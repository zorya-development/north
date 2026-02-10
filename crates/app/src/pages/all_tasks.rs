use leptos::prelude::*;

use crate::components::task_list::TaskList;
use crate::server_fns::tasks::{get_all_tasks, get_completed_tasks};
use crate::stores::task_store::TaskStore;

#[component]
pub fn AllTasksPage() -> impl IntoView {
    let all_tasks = Resource::new(|| (), |_| get_all_tasks());
    let completed = Resource::new(|| (), |_| get_completed_tasks(None, false));
    let store = TaskStore::new(all_tasks);

    view! {
        <div class="space-y-4">
            <h1 class="text-xl font-semibold text-text-primary">
                "All Tasks"
            </h1>
            <TaskList
                resource=all_tasks
                store=store
                empty_message="No tasks yet."
                completed_resource=completed
            />
        </div>
    }
}
