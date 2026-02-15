use leptos::prelude::*;
use north_domain::TaskWithMeta;
use north_ui::Spinner;

use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::containers::task_inline_form::TaskInlineForm;
use crate::containers::task_list_item::TaskListItem;

#[component]
pub fn InboxView(
    active_task_ids: Memo<Vec<i64>>,
    completed_task_ids: Memo<Vec<i64>>,
    completed_count: Memo<usize>,
    is_loaded: Signal<bool>,
    is_form_open: ReadSignal<bool>,
    set_form_open: WriteSignal<bool>,
    on_task_click: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    active_tasks_for_reorder: Memo<Vec<TaskWithMeta>>,
) -> impl IntoView {
    let (showing_completed, set_showing_completed) = signal(false);
    let drag_ctx = use_context::<DragDropContext>();

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-semibold tracking-tight text-text-primary">
                    "Inbox"
                </h1>
                <button
                    on:click=move |_| set_form_open.set(!is_form_open.get_untracked())
                    class="text-sm text-text-secondary hover:text-accent \
                           transition-colors cursor-pointer"
                >
                    "+" " Add task"
                </button>
            </div>

            <Show when=move || is_form_open.get()>
                <TaskInlineForm on_done=Callback::new(move |()| {
                    set_form_open.set(false)
                })/>
            </Show>

            <div
                on:drop=move |ev: web_sys::DragEvent| {
                    ev.prevent_default();
                    let current_tasks = active_tasks_for_reorder.get_untracked();
                    handle_drop(&ev, drag_ctx, &current_tasks, on_reorder);
                }
                on:dragover=move |ev: web_sys::DragEvent| {
                    if drag_ctx.is_some() {
                        ev.prevent_default();
                    }
                }
            >
                {move || {
                    if !is_loaded.get() {
                        return view! { <Spinner/> }.into_any();
                    }
                    let ids = active_task_ids.get();
                    if ids.is_empty() {
                        view! {
                            <div class="text-sm text-text-secondary py-8 text-center">
                                "No tasks in your inbox. Add one above."
                            </div>
                        }
                            .into_any()
                    } else {
                        ids.into_iter()
                            .map(|id| {
                                view! {
                                    <TaskListItem
                                        task_id=id
                                        draggable=true
                                        on_click=Callback::new(move |id| {
                                            on_task_click.run(id)
                                        })
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
                                    set_showing_completed.update(|v| *v = !*v);
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
                                        completed_task_ids
                                            .get()
                                            .into_iter()
                                            .map(|id| {
                                                view! {
                                                    <TaskListItem task_id=id/>
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
