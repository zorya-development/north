use std::collections::BTreeMap;

use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::task_card::TaskCard;
use crate::components::task_detail_modal::{TaskDetailContext, TaskDetailModal};
use crate::server_fns::tasks::get_today_tasks;
use crate::stores::task_store::TaskStore;

#[component]
pub fn TodayPage() -> impl IntoView {
    let open_task_id = RwSignal::new(None::<i64>);
    provide_context(TaskDetailContext { open_task_id });

    let today_tasks = Resource::new(|| (), |_| get_today_tasks());
    let store = TaskStore::new(today_tasks);

    let task_ids = Signal::derive(move || {
        today_tasks
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .iter()
            .map(|t| t.task.id)
            .collect::<Vec<_>>()
    });

    let modal_store = store.clone();

    view! {
        <div class="space-y-4">
            <h1 class="text-2xl font-semibold tracking-tight text-text-primary">"Today"</h1>
            <Suspense fallback=move || {
                view! {
                    <div class="text-sm text-text-secondary py-4">
                        "Loading tasks..."
                    </div>
                }
            }>
                {
                    let store = store.clone();
                    move || {
                        let store = store.clone();
                        Suspend::new(async move {
                            match today_tasks.await {
                                Ok(tasks) => {
                                    if tasks.is_empty() {
                                        view! {
                                            <div class="text-sm text-text-secondary \
                                                        py-8 text-center">
                                                "No tasks scheduled for today."
                                            </div>
                                        }
                                        .into_any()
                                    } else {
                                        let groups = group_by_project(tasks);
                                        view! {
                                            <div class="space-y-4">
                                                {groups
                                                    .into_iter()
                                                    .map(|(label, tasks)| {
                                                        let store = store.clone();
                                                        view! {
                                                            <div>
                                                                <h2 class="text-xs font-medium \
                                                                           text-text-secondary \
                                                                           uppercase \
                                                                           tracking-wide \
                                                                           px-3 pb-1">
                                                                    {label}
                                                                </h2>
                                                                {tasks
                                                                    .into_iter()
                                                                    .map(|task| {
                                                                        view! {
                                                                            <TaskCard
                                                                                task=task
                                                                                on_toggle_complete=store.on_toggle_complete
                                                                                on_delete=store.on_delete
                                                                                on_update=store.on_update
                                                                                on_set_start_at=store.on_set_start_at
                                                                                on_clear_start_at=store.on_clear_start_at
                                                                                on_set_project=store.on_set_project
                                                                                on_clear_project=store.on_clear_project
                                                                                on_set_tags=store.on_set_tags
                                                                                on_review=store.on_review
                                                                            />
                                                                        }
                                                                    })
                                                                    .collect::<Vec<_>>()}
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </div>
                                        }
                                        .into_any()
                                    }
                                }
                                Err(e) => {
                                    view! {
                                        <div class="text-sm text-danger py-4">
                                            {format!("Failed to load tasks: {e}")}
                                        </div>
                                    }
                                    .into_any()
                                }
                            }
                        })
                    }
                }
            </Suspense>
            <TaskDetailModal task_ids=task_ids task_store=modal_store/>
        </div>
    }
}

fn group_by_project(tasks: Vec<TaskWithMeta>) -> Vec<(String, Vec<TaskWithMeta>)> {
    let mut groups: BTreeMap<String, Vec<TaskWithMeta>> = BTreeMap::new();
    for task in tasks {
        let label = task
            .project_title
            .clone()
            .unwrap_or_else(|| "No Project".to_string());
        groups.entry(label).or_default().push(task);
    }
    groups.into_iter().collect()
}
