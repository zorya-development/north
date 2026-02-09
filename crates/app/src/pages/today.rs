use leptos::prelude::*;

use crate::components::task_list::TaskList;
use crate::server_fns::tasks::get_today_tasks;
use crate::stores::task_store::TaskStore;

#[component]
pub fn TodayPage() -> impl IntoView {
    let today_tasks = Resource::new(|| (), |_| get_today_tasks());
    let store = TaskStore::new(today_tasks);

    view! {
        <div class="space-y-4">
            <h1 class="text-xl font-semibold text-text-primary">"Today"</h1>
            <TaskList
                resource=today_tasks
                store=store
                empty_message="No tasks scheduled for today."
            />
        </div>
    }
}
