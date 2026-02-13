use std::collections::HashSet;

use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::components::task_card::TaskCard;
use crate::server_fns::tasks::get_subtasks;

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
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    #[prop(optional)] completed_resource: Option<
        Resource<Result<Vec<TaskWithMeta>, ServerFnError>>,
    >,
    #[prop(default = false)] draggable: bool,
) -> impl IntoView {
    let (showing_completed, set_showing_completed) = signal(false);
    let expanded = RwSignal::new(HashSet::<i64>::new());
    let drag_ctx = use_context::<DragDropContext>();

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
                                let tasks_clone = tasks.clone();
                                view! {
                                    <div
                                        on:drop=move |ev: web_sys::DragEvent| {
                                            ev.prevent_default();
                                            handle_drop(
                                                &ev,
                                                drag_ctx,
                                                &tasks_clone,
                                                on_reorder,
                                            );
                                        }
                                        on:dragover=move |ev: web_sys::DragEvent| {
                                            if drag_ctx.is_some() {
                                                ev.prevent_default();
                                            }
                                        }
                                    >
                                        {tasks
                                            .into_iter()
                                            .map(|task| {
                                                let task_id = task.task.id;
                                                let has_subtasks =
                                                    task.subtask_count > 0;
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
                                                        draggable=draggable
                                                    />
                                                    {if has_subtasks {
                                                        Some(view! {
                                                            <InlineSubtasks
                                                                parent_id=task_id
                                                                expanded=expanded
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
                                                                draggable=draggable
                                                                depth=1
                                                            />
                                                        })
                                                    } else {
                                                        None
                                                    }}
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

    // Don't drop on self
    if dragging_id == target_id {
        ctx.dragging_task_id.set(None);
        ctx.drop_target.set(None);
        return;
    }

    // Find target task's index in list
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
            // Make the dragged task a child of the target
            let new_key = north_domain::sort_key_after(None);
            on_reorder.run((dragging_id, new_key, Some(Some(target_id))));
        }
    }

    ctx.dragging_task_id.set(None);
    ctx.drop_target.set(None);
}

#[component]
fn InlineSubtasks(
    parent_id: i64,
    expanded: RwSignal<HashSet<i64>>,
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
    #[prop(default = false)] draggable: bool,
    #[prop(default = 1)] depth: u8,
) -> impl IntoView {
    let is_expanded = Signal::derive(move || expanded.get().contains(&parent_id));

    view! {
        <div class="ml-2">
            <button
                class="text-xs text-text-secondary hover:text-text-primary \
                       transition-colors flex items-center gap-1 px-4 py-1"
                on:click=move |_| {
                    expanded.update(|set| {
                        if set.contains(&parent_id) {
                            set.remove(&parent_id);
                        } else {
                            set.insert(parent_id);
                        }
                    });
                }
            >
                <north_ui::Icon
                    kind={if is_expanded.get() {
                        north_ui::IconKind::ChevronDown
                    } else {
                        north_ui::IconKind::ChevronRight
                    }}
                    class="w-3 h-3"
                />
                {move || {
                    if is_expanded.get() {
                        "Hide subtasks"
                    } else {
                        "Show subtasks"
                    }
                }}
            </button>
            <Show when=move || is_expanded.get()>
                <SubtaskList
                    parent_id=parent_id
                    expanded=expanded
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
                    draggable=draggable
                    depth=depth
                />
            </Show>
        </div>
    }
}

#[component]
fn SubtaskList(
    parent_id: i64,
    expanded: RwSignal<HashSet<i64>>,
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
    #[prop(default = false)] draggable: bool,
    #[prop(default = 1)] depth: u8,
) -> impl IntoView {
    let subtasks = Resource::new(
        move || parent_id,
        |pid| get_subtasks(pid),
    );

    view! {
        <Suspense fallback=|| ()>
            {move || {
                Suspend::new(async move {
                    match subtasks.await {
                        Ok(tasks) => {
                            view! {
                                <div>
                                    {tasks
                                        .into_iter()
                                        .map(|task| {
                                            let task_id = task.task.id;
                                            let has_subtasks =
                                                task.subtask_count > 0
                                                    && depth < 2;
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
                                                    draggable=draggable
                                                    depth=depth
                                                />
                                                {if has_subtasks {
                                                    Some(view! {
                                                        <InlineSubtasks
                                                            parent_id=task_id
                                                            expanded=expanded
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
                                                            draggable=draggable
                                                            depth={depth + 1}
                                                        />
                                                    })
                                                } else {
                                                    None
                                                }}
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>
                            }
                            .into_any()
                        }
                        Err(_) => view! { <div/> }.into_any(),
                    }
                })
            }}
        </Suspense>
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
