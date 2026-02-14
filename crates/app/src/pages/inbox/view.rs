use leptos::prelude::*;
use north_domain::TaskWithMeta;

use super::controller::InboxController;
use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::containers::task_inline_form::TaskInlineForm;
use crate::containers::task_list_item::TaskListItem;

#[component]
pub fn InboxView(ctrl: InboxController) -> impl IntoView {
    let (showing_completed, set_showing_completed) = signal(false);
    let (is_form_open, set_form_open) = ctrl.is_new_task_form_open;
    let drag_ctx = use_context::<DragDropContext>();
    let active_tasks_for_reorder = ctrl.active_tasks_for_reorder();

    view! {
        <div class="space-y-4">
            <h1 class="text-2xl font-semibold tracking-tight text-text-primary">
                "Inbox"
            </h1>

            <Show
                when=move || is_form_open.get()
                fallback=move || {
                    view! {
                        <button
                            on:click=move |_| set_form_open.set(true)
                            class="flex items-center gap-2 p-4 w-full text-left \
                                   cursor-pointer border-2 border-border rounded-xl \
                                   hover:border-accent transition-colors"
                        >
                            <span class="text-accent text-lg font-medium">"+"</span>
                            <span class="text-sm text-text-secondary">
                                "Add a task..."
                            </span>
                        </button>
                    }
                }
            >
                <TaskInlineForm on_done=Callback::new(move |()| {
                    set_form_open.set(false)
                })/>
            </Show>

            <div
                on:drop=move |ev: web_sys::DragEvent| {
                    ev.prevent_default();
                    let current_tasks = active_tasks_for_reorder.get_untracked();
                    handle_drop(&ev, drag_ctx, &current_tasks, &ctrl);
                }
                on:dragover=move |ev: web_sys::DragEvent| {
                    if drag_ctx.is_some() {
                        ev.prevent_default();
                    }
                }
            >
                {move || {
                    let ids = ctrl.active_task_ids.get();
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
                                            ctrl.open_detail(id)
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
                let count = ctrl.completed_count.get();
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
                                        ctrl.completed_task_ids
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
    ctrl: &InboxController,
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
            ctrl.reorder_task(dragging_id, new_key, Some(None));
        }
        DropZone::Below => {
            let above_key = Some(tasks[target_idx].task.sort_key.as_str());
            let below_key = if target_idx + 1 < tasks.len() {
                Some(tasks[target_idx + 1].task.sort_key.as_str())
            } else {
                None
            };
            let new_key = north_domain::sort_key_between(above_key, below_key);
            ctrl.reorder_task(dragging_id, new_key, Some(None));
        }
        DropZone::Nest => {
            let new_key = north_domain::sort_key_after(None);
            ctrl.reorder_task(dragging_id, new_key, Some(Some(target_id)));
        }
    }

    ctx.dragging_task_id.set(None);
    ctx.drop_target.set(None);
}
