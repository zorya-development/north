use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::task_card::TaskCard;

#[component]
pub fn TaskListView(
    resource: Resource<Result<Vec<TaskWithMeta>, ServerFnError>>,
    on_toggle_complete: Callback<(i64, bool)>,
    on_delete: Callback<i64>,
    on_update: Callback<(i64, String, Option<String>)>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    on_review: Callback<i64>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
) -> impl IntoView {
    view! {
        <Suspense fallback=move || {
            view! {
                <div class="text-sm text-text-secondary py-4">
                    "Loading tasks..."
                </div>
            }
        }>
            {move || {
                Suspend::new(async move {
                    match resource.await {
                        Ok(tasks) => {
                            if tasks.is_empty() {
                                view! {
                                    <div class="text-sm text-text-secondary \
                                                py-8 text-center">
                                        {empty_message}
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div>
                                        {tasks
                                            .into_iter()
                                            .map(|task| {
                                                view! {
                                                    <TaskCard
                                                        task=task
                                                        on_toggle_complete=on_toggle_complete
                                                        on_delete=on_delete
                                                        on_update=on_update
                                                        on_set_start_at=on_set_start_at
                                                        on_clear_start_at=on_clear_start_at
                                                        on_set_project=on_set_project
                                                        on_clear_project=on_clear_project
                                                        on_review=on_review
                                                        show_review=show_review
                                                    />
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
                                <div class="text-sm text-red-500 py-4">
                                    {format!("Failed to load tasks: {e}")}
                                </div>
                            }
                                .into_any()
                        }
                    }
                })
            }}
        </Suspense>
    }
}
