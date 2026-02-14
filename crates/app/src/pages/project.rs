use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::drag_drop::DragDropContext;
use crate::components::task_detail_modal::{TaskDetailContext, TaskDetailModal};
use crate::components::task_form::InlineTaskForm;
use crate::components::task_list::TaskList;
use crate::server_fns::projects::{get_project, get_project_tasks, set_task_project};
use crate::server_fns::tasks::{create_task, get_completed_tasks};
use crate::stores::task_store::TaskStore;

#[component]
pub fn ProjectPage() -> impl IntoView {
    let open_task_id = RwSignal::new(None::<i64>);
    provide_context(TaskDetailContext { open_task_id });
    provide_context(DragDropContext::new());

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
    let completed = Resource::new(project_id, |id| get_completed_tasks(Some(id), false));
    let store = TaskStore::new(project_tasks);

    let create_action = Action::new(move |input: &(String, Option<String>, i64)| {
        let (title, body, pid) = input.clone();
        async move {
            let task = create_task(title, body).await?;
            set_task_project(task.id, pid).await?;
            Ok::<_, ServerFnError>(())
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = create_action.value().get() {
            project_tasks.refetch();
        }
    });

    let task_ids = Signal::derive(move || {
        project_tasks
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .iter()
            .map(|t| t.task.id)
            .collect::<Vec<_>>()
    });

    view! {
        <div class="space-y-4">
            <Suspense fallback=move || {
                view! {
                    <h1 class="text-2xl font-semibold tracking-tight text-text-primary">
                        "Loading..."
                    </h1>
                }
            }>
                {move || {
                    Suspend::new(async move {
                        match project.await {
                            Ok(p) => view! {
                                <h1 class="text-2xl font-semibold tracking-tight text-text-primary">
                                    {p.title}
                                </h1>
                            }
                            .into_any(),
                            Err(e) => view! {
                                <h1 class="text-2xl font-semibold tracking-tight text-danger">
                                    {format!("Error: {e}")}
                                </h1>
                            }
                            .into_any(),
                        }
                    })
                }}
            </Suspense>
            <InlineTaskForm on_submit={
                let create_action = create_action.clone();
                move |title: String, body: Option<String>| {
                    let pid = project_id();
                    create_action.dispatch((title, body, pid));
                }
            }/>
            <TaskList
                resource=project_tasks
                store=store.clone()
                show_project=false
                empty_message="No tasks in this project."
                completed_resource=completed
                draggable=true
            />
            <TaskDetailModal task_ids=task_ids task_store=store/>
        </div>
    }
}
