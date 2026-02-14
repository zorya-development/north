use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::components::task_card::TaskCard;
use crate::components::task_form::InlineTaskForm;

#[component]
pub fn InboxView(
    tasks: Memo<Vec<TaskWithMeta>>,
    on_toggle_complete: Callback<(i64, bool)>,
    on_delete: Callback<i64>,
    on_update: Callback<(i64, String, Option<String>)>,
    on_create: Callback<(String, Option<String>)>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    on_set_tags: Callback<(i64, Vec<String>)>,
    on_review: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
) -> impl IntoView {
    let (showing_completed, set_showing_completed) = signal(false);
    let drag_ctx = use_context::<DragDropContext>();

    let active_tasks = Memo::new(move |_| {
        tasks
            .get()
            .into_iter()
            .filter(|t| t.task.completed_at.is_none())
            .collect::<Vec<_>>()
    });

    let completed_tasks = Memo::new(move |_| {
        tasks
            .get()
            .into_iter()
            .filter(|t| t.task.completed_at.is_some())
            .collect::<Vec<_>>()
    });

    let completed_count = Memo::new(move |_| completed_tasks.get().len());

    let on_form_submit = move |title: String, body: Option<String>| {
        on_create.run((title, body));
    };

    view! {
        <div class="space-y-4">
            <h1 class="text-2xl font-semibold tracking-tight text-text-primary">
                "Inbox"
            </h1>
            <InlineTaskForm on_submit=on_form_submit/>
            <div
                on:drop=move |ev: web_sys::DragEvent| {
                    ev.prevent_default();
                    let current_tasks = active_tasks.get_untracked();
                    handle_drop(&ev, drag_ctx, &current_tasks, on_reorder);
                }
                on:dragover=move |ev: web_sys::DragEvent| {
                    if drag_ctx.is_some() {
                        ev.prevent_default();
                    }
                }
            >
                {move || {
                    let current = active_tasks.get();
                    if current.is_empty() {
                        view! {
                            <div class="text-sm text-text-secondary py-8 text-center">
                                "No tasks in your inbox. Add one above."
                            </div>
                        }
                            .into_any()
                    } else {
                        current
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
                                        draggable=true
                                    />
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>

            // Completed section
            {move || {
                let count = completed_count.get();
                if count == 0 {
                    None
                } else {
                    Some(view! {
                        <div class="mt-4 border-t border-border pt-3">
                            <button
                                class="text-xs text-text-secondary \
                                       hover:text-text-primary transition-colors"
                                on:click=move |_| {
                                    set_showing_completed
                                        .update(|v| *v = !*v);
                                }
                            >
                                {move || {
                                    if showing_completed.get() {
                                        format!("Hide completed ({count})")
                                    } else {
                                        format!("Show completed ({count})")
                                    }
                                }}
                            </button>
                            <Show when=move || showing_completed.get()>
                                <div class="mt-2 opacity-60">
                                    {move || {
                                        completed_tasks
                                            .get()
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
                                                    />
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </div>
                            </Show>
                        </div>
                    })
                }
            }}
        </div>
    }
}

fn handle_drop(
    _ev: &web_sys::DragEvent,
    drag_ctx: Option<DragDropContext>,
    tasks: &[TaskWithMeta],
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
) {
    let Some(ctx) = drag_ctx else { return };
    let Some(dragging_id) = ctx.dragging_task_id.get_untracked() else {
        return;
    };
    let Some((target_id, zone)) = ctx.drop_target.get_untracked() else {
        return;
    };

    if dragging_id == target_id {
        ctx.dragging_task_id.set(None);
        ctx.drop_target.set(None);
        return;
    }

    let target_idx = tasks.iter().position(|t| t.task.id == target_id);
    let Some(target_idx) = target_idx else {
        ctx.dragging_task_id.set(None);
        ctx.drop_target.set(None);
        return;
    };

    match zone {
        DropZone::Above => {
            let above_key = if target_idx > 0 {
                Some(tasks[target_idx - 1].task.sort_key.as_str())
            } else {
                None
            };
            let below_key = Some(tasks[target_idx].task.sort_key.as_str());
            let new_key = north_domain::sort_key_between(above_key, below_key);
            on_reorder.run((dragging_id, new_key, Some(None)));
        }
        DropZone::Below => {
            let above_key = Some(tasks[target_idx].task.sort_key.as_str());
            let below_key = if target_idx + 1 < tasks.len() {
                Some(tasks[target_idx + 1].task.sort_key.as_str())
            } else {
                None
            };
            let new_key = north_domain::sort_key_between(above_key, below_key);
            on_reorder.run((dragging_id, new_key, Some(None)));
        }
        DropZone::Nest => {
            let new_key = north_domain::sort_key_after(None);
            on_reorder.run((dragging_id, new_key, Some(Some(target_id))));
        }
    }

    ctx.dragging_task_id.set(None);
    ctx.drop_target.set(None);
}
