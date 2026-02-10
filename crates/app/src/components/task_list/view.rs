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
    on_set_tags: Callback<(i64, Vec<String>)>,
    on_review: Callback<i64>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    #[prop(optional)] completed_resource: Option<
        Resource<Result<Vec<TaskWithMeta>, ServerFnError>>,
    >,
) -> impl IntoView {
    let (showing_completed, set_showing_completed) = signal(false);

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
                                                        on_set_tags=on_set_tags
                                                        on_review=on_review
                                                        show_review=show_review
                                                        show_project=show_project
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

        {move || {
            completed_resource.map(|cr| {
                view! {
                    <CompletedSection
                        resource=cr
                        showing=showing_completed
                        set_showing=set_showing_completed
                        on_toggle_complete=on_toggle_complete
                        on_delete=on_delete
                        on_update=on_update
                        on_set_start_at=on_set_start_at
                        on_clear_start_at=on_clear_start_at
                        on_set_project=on_set_project
                        on_clear_project=on_clear_project
                        on_set_tags=on_set_tags
                        on_review=on_review
                        show_project=show_project
                    />
                }
            })
        }}
    }
}

#[component]
fn CompletedSection(
    resource: Resource<Result<Vec<TaskWithMeta>, ServerFnError>>,
    showing: ReadSignal<bool>,
    set_showing: WriteSignal<bool>,
    on_toggle_complete: Callback<(i64, bool)>,
    on_delete: Callback<i64>,
    on_update: Callback<(i64, String, Option<String>)>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    on_set_tags: Callback<(i64, Vec<String>)>,
    on_review: Callback<i64>,
    #[prop(default = true)] show_project: bool,
) -> impl IntoView {
    view! {
        <Suspense fallback=|| ()>
            {move || {
                Suspend::new(async move {
                    match resource.await {
                        Ok(tasks) if !tasks.is_empty() => {
                            let count = tasks.len();
                            view! {
                                <div class="mt-4 border-t border-border pt-3">
                                    <button
                                        class="text-xs text-text-secondary \
                                               hover:text-text-primary \
                                               transition-colors"
                                        on:click=move |_| {
                                            set_showing.update(|v| *v = !*v);
                                        }
                                    >
                                        {move || {
                                            if showing.get() {
                                                format!("Hide completed ({count})")
                                            } else {
                                                format!("Show completed ({count})")
                                            }
                                        }}
                                    </button>
                                    <Show when=move || showing.get()>
                                        <div class="mt-2 opacity-60">
                                            {tasks
                                                .clone()
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
                                                            on_set_tags=on_set_tags
                                                            on_review=on_review
                                                            show_project=show_project
                                                        />
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    </Show>
                                </div>
                            }
                            .into_any()
                        }
                        _ => view! { <div/> }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}
