use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::task_list::TaskList;
use crate::server_fns::projects::{get_project, get_project_tasks};
use crate::server_fns::tasks::get_completed_tasks;
use crate::stores::task_store::TaskStore;

#[component]
pub fn ProjectPage() -> impl IntoView {
    let params = use_params_map();

    let project_id = move || {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(0)
    };

    let project = Resource::new(project_id, |id| get_project(id));
    let project_tasks = Resource::new(project_id, |id| get_project_tasks(id));
    let completed = Resource::new(project_id, |id| {
        get_completed_tasks(Some(id), false)
    });
    let store = TaskStore::new(project_tasks);

    view! {
        <div class="space-y-4">
            <Suspense fallback=move || {
                view! {
                    <h1 class="text-xl font-semibold text-text-primary">
                        "Loading..."
                    </h1>
                }
            }>
                {move || {
                    Suspend::new(async move {
                        match project.await {
                            Ok(p) => view! {
                                <h1 class="text-xl font-semibold text-text-primary">
                                    {p.title}
                                </h1>
                            }
                            .into_any(),
                            Err(e) => view! {
                                <h1 class="text-xl font-semibold text-red-500">
                                    {format!("Error: {e}")}
                                </h1>
                            }
                            .into_any(),
                        }
                    })
                }}
            </Suspense>
            <TaskList
                resource=project_tasks
                store=store
                show_project=false
                empty_message="No tasks in this project."
                completed_resource=completed
            />
        </div>
    }
}
