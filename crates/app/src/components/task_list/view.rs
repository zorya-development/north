use leptos::prelude::*;
use north_domain::Task;
use north_ui::Spinner;

use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::containers::task_list_item::TaskListItem;

#[component]
pub fn TaskListView(
    active_task_ids: Memo<Vec<i64>>,
    active_tasks_for_reorder: Memo<Vec<Task>>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    #[prop(default = false)] compact: bool,
    #[prop(default = false)] draggable: bool,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    on_task_click: Callback<i64>,
    is_loaded: Signal<bool>,
) -> impl IntoView {
    let drag_ctx = use_context::<DragDropContext>();

    view! {
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
                            {empty_message}
                        </div>
                    }
                        .into_any()
                } else {
                    ids.into_iter()
                        .map(|id| {
                            view! {
                                <TaskListItem
                                    task_id=id
                                    show_review=show_review
                                    show_project=show_project
                                    draggable=draggable
                                    compact=compact
                                    on_click=on_task_click
                                />
                            }
                        })
                        .collect_view()
                        .into_any()
                }
            }}
        </div>
    }
}

fn handle_drop(
    _ev: &web_sys::DragEvent,
    drag_ctx: Option<DragDropContext>,
    tasks: &[Task],
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

    let target_idx = tasks.iter().position(|t| t.id == target_id);
    let Some(target_idx) = target_idx else {
        ctx.dragging_task_id.set(None);
        ctx.drop_target.set(None);
        return;
    };

    match zone {
        DropZone::Above => {
            let above_key = if target_idx > 0 {
                Some(tasks[target_idx - 1].sort_key.as_str())
            } else {
                None
            };
            let below_key = Some(tasks[target_idx].sort_key.as_str());
            let new_key = north_domain::sort_key_between(above_key, below_key);
            on_reorder.run((dragging_id, new_key, Some(None)));
        }
        DropZone::Below => {
            let above_key = Some(tasks[target_idx].sort_key.as_str());
            let below_key = if target_idx + 1 < tasks.len() {
                Some(tasks[target_idx + 1].sort_key.as_str())
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
