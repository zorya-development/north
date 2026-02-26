use leptos::prelude::*;
use north_stores::{TaskModel, TaskStoreFilter};
use north_ui::Spinner;
use wasm_bindgen::JsCast;

use super::controller::TraversableTaskListController;
use super::tree::*;
use crate::atoms::{Text, TextColor, TextTag, TextVariant};
use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::components::mirror_overlay::MirrorOverlay;
use crate::containers::task_list_item::{ItemConfig, TaskListItem};

#[component]
pub fn TraversableTaskListView(
    ctrl: TraversableTaskListController,
    #[prop(default = ItemConfig::default())] item_config: ItemConfig,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    is_loaded: Signal<bool>,
    #[prop(default = false)] scoped: bool,
) -> impl IntoView {
    let flat_nodes = ctrl.flat_nodes;
    let cursor_task_id = ctrl.cursor_task_id;
    let inline_mode = ctrl.inline_mode;
    let create_input_value = ctrl.create_input_value;
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let drag_ctx = use_context::<DragDropContext>();
    let app_store = north_stores::use_app_store();
    let all_tasks_for_drop = app_store.tasks.filtered(TaskStoreFilter::default());

    if !scoped {
        // Global keyboard listener — works regardless of focus.
        // Skips when an input/textarea is focused. Modal check is in the controller.
        window_event_listener(leptos::ev::keydown, move |ev| {
            if let Some(el) = document().active_element() {
                if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                    let tag = html_el.tag_name().to_lowercase();
                    if tag == "input" || tag == "textarea" || html_el.is_content_editable() {
                        return;
                    }
                }
            }
            ctrl.handle_keydown(&ev);
        });
    }

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if !scoped {
            return;
        }
        if let Some(el) = document().active_element() {
            if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                let tag = html_el.tag_name().to_lowercase();
                if tag == "input" || tag == "textarea" || html_el.is_content_editable() {
                    return;
                }
            }
        }
        ctrl.handle_keydown(&ev);
    };

    let on_container_click = move |_: web_sys::MouseEvent| {
        if scoped {
            if let Some(el) = container_ref.get() {
                let _ = el.focus();
            }
        }
    };

    let on_focus = move |_: web_sys::FocusEvent| {
        if scoped && cursor_task_id.get_untracked().is_none() {
            if let Some(first) = flat_nodes.get_untracked().first() {
                cursor_task_id.set(Some(first.task_id));
            }
        }
    };

    view! {
        <div
            node_ref=container_ref
            tabindex=if scoped { "0" } else { "-1" }
            class=if scoped { "no-focus-ring" } else { "" }
            on:keydown=on_keydown
            on:click=on_container_click
            on:focus=on_focus
            on:drop=move |ev: web_sys::DragEvent| {
                ev.prevent_default();
                let nodes = flat_nodes.get_untracked();
                let tasks = all_tasks_for_drop.get_untracked();
                handle_drop(drag_ctx, &nodes, &tasks, ctrl);
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
                let nodes = flat_nodes.get();
                if nodes.is_empty() {
                    return view! {
                        <Show when=move || {
                            matches!(inline_mode.get(), InlineMode::CreateTop)
                        }>
                            <InlineCreateInput
                                depth=Memo::new(|_| 0u8)
                                value=create_input_value
                                ctrl=ctrl
                            />
                        </Show>
                        <Show when=move || {
                            !matches!(inline_mode.get(), InlineMode::CreateTop)
                        }>
                            <div data-testid="empty-task-list">
                                <Text
                                    variant=TextVariant::BodyMd
                                    color=TextColor::Secondary
                                    tag=TextTag::P
                                    class="py-8 text-center"
                                >
                                    {empty_message}
                                </Text>
                            </div>
                        </Show>
                    }
                    .into_any();
                }
                view! {
                    <Show when=move || {
                        matches!(inline_mode.get(), InlineMode::CreateTop)
                    }>
                        <InlineCreateInput
                            depth=Memo::new(|_| 0u8)
                            value=create_input_value
                            ctrl=ctrl
                        />
                    </Show>
                    <div data-testid="task-list">
                    <For
                        each=move || flat_nodes.get()
                        key=|node| node.task_id
                        children=move |node| {
                            let task_id = node.task_id;
                            let initial_depth = node.depth;

                            let depth = Memo::new(move |_| {
                                flat_nodes
                                    .get()
                                    .iter()
                                    .find(|n| n.task_id == task_id)
                                    .map(|n| n.depth)
                                    .unwrap_or(initial_depth)
                            });

                            let is_selected = Memo::new(move |_| {
                                cursor_task_id.get() == Some(task_id)
                            });

                            let is_editing = Memo::new(move |_| {
                                matches!(
                                    inline_mode.get(),
                                    InlineMode::Edit { task_id: id }
                                    if id == task_id
                                )
                            });

                            let has_create_before = Memo::new(move |_| {
                                matches!(
                                    inline_mode.get(),
                                    InlineMode::Create {
                                        anchor_task_id,
                                        placement: Placement::Before,
                                        ..
                                    } if anchor_task_id == task_id
                                )
                            });

                            let has_create_after = Memo::new(move |_| {
                                matches!(
                                    inline_mode.get(),
                                    InlineMode::Create {
                                        anchor_task_id,
                                        placement: Placement::After,
                                        ..
                                    } if anchor_task_id == task_id
                                )
                            });

                            let create_depth = Memo::new(move |_| {
                                match inline_mode.get() {
                                    InlineMode::Create { depth, .. } => depth,
                                    _ => 0,
                                }
                            });

                            view! {
                                <Show when=move || has_create_before.get()>
                                    <InlineCreateInput
                                        depth=create_depth
                                        value=create_input_value
                                        ctrl=ctrl
                                    />
                                </Show>

                                <Show when=move || !is_editing.get()>
                                    <div
                                        data-testid="task-row"
                                        data-task-id=task_id
                                        data-focused=move || is_selected.get().to_string()
                                        style=move || {
                                            format!(
                                                "padding-left: {}rem",
                                                depth.get() as f32 * 1.5,
                                            )
                                        }
                                        class=move || {
                                            if is_selected.get() {
                                                "trash-polka-focus"
                                            } else {
                                                ""
                                            }
                                        }
                                        on:click=move |_| {
                                            cursor_task_id.set(Some(task_id));
                                            ctrl.open_detail_for(task_id);
                                        }
                                    >
                                        <TaskListItem
                                            task_id=task_id
                                            config=item_config
                                        />
                                    </div>
                                </Show>

                                <Show when=move || is_editing.get()>
                                    <InlineEditInput
                                        task_id=task_id
                                        depth=depth
                                        ctrl=ctrl
                                    />
                                </Show>

                                <Show when=move || has_create_after.get()>
                                    <InlineCreateInput
                                        depth=create_depth
                                        value=create_input_value
                                        ctrl=ctrl
                                    />
                                </Show>
                            }
                        }
                    />
                    </div>
                }
                .into_any()
            }}
        </div>
    }
}

fn handle_drop(
    drag_ctx: Option<DragDropContext>,
    flat_nodes: &[FlatNode],
    all_tasks: &[TaskModel],
    ctrl: TraversableTaskListController,
) {
    let Some(ctx) = drag_ctx else { return };
    let Some(dragging_id) = ctx.dragging_task_id.get_untracked() else {
        return;
    };
    let Some((target_id, zone)) = ctx.drop_target.get_untracked() else {
        return;
    };

    // No-op: dropped on itself.
    if dragging_id == target_id {
        ctx.dragging_task_id.set(None);
        ctx.drop_target.set(None);
        return;
    }

    // Prevent cycles: cannot drop a parent onto its own descendant.
    if is_descendant_of(flat_nodes, dragging_id, target_id) {
        ctx.dragging_task_id.set(None);
        ctx.drop_target.set(None);
        return;
    }

    let target_node = flat_nodes.iter().find(|n| n.task_id == target_id);
    let Some(target_node) = target_node else {
        ctx.dragging_task_id.set(None);
        ctx.drop_target.set(None);
        return;
    };

    match zone {
        DropZone::Above => {
            let parent_id = target_node.parent_id;
            let siblings: Vec<i64> = flat_nodes
                .iter()
                .filter(|n| n.parent_id == parent_id && !n.is_completed)
                .map(|n| n.task_id)
                .collect();
            let pos = siblings.iter().position(|&id| id == target_id);
            let above_key = pos
                .filter(|&p| p > 0)
                .and_then(|p| task_sort_key(all_tasks, siblings[p - 1]));
            let below_key = task_sort_key(all_tasks, target_id);
            let new_key = north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref());
            ctrl.reorder_task(dragging_id, new_key, Some(parent_id));
        }
        DropZone::Below => {
            let parent_id = target_node.parent_id;
            let siblings: Vec<i64> = flat_nodes
                .iter()
                .filter(|n| n.parent_id == parent_id && !n.is_completed)
                .map(|n| n.task_id)
                .collect();
            let pos = siblings.iter().position(|&id| id == target_id);
            let above_key = task_sort_key(all_tasks, target_id);
            let below_key = pos.and_then(|p| {
                siblings
                    .get(p + 1)
                    .and_then(|&id| task_sort_key(all_tasks, id))
            });
            let new_key = north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref());
            ctrl.reorder_task(dragging_id, new_key, Some(parent_id));
        }
        DropZone::Nest => {
            // Become last child of target.
            let last_child_key = flat_nodes
                .iter()
                .filter(|n| n.parent_id == Some(target_id) && !n.is_completed)
                .filter_map(|n| task_sort_key(all_tasks, n.task_id))
                .next_back();
            let new_key = north_dto::sort_key_after(last_child_key.as_deref());
            ctrl.reorder_task(dragging_id, new_key, Some(Some(target_id)));
        }
    }

    ctx.dragging_task_id.set(None);
    ctx.drop_target.set(None);
}

/// Borderless inline input for editing an existing task title.
#[component]
fn InlineEditInput(
    task_id: i64,
    depth: Memo<u8>,
    ctrl: TraversableTaskListController,
) -> impl IntoView {
    let app_store = north_stores::use_app_store();
    let initial_title = app_store
        .tasks
        .get_by_id(task_id)
        .get_untracked()
        .map(|t| t.title.clone())
        .unwrap_or_default();
    let (value, set_value) = signal(initial_title);
    let input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move || {
        if let Some(el) = input_ref.get() {
            let _ = el.focus();
            let len = el.value().len() as u32;
            let _ = el.set_selection_range(0, len);
        }
    });

    view! {
        <div
            style=move || {
                format!("padding-left: {}rem", depth.get() as f32 * 1.5)
            }
            class="pr-4 py-1 trash-polka-focus"
        >
            <div class="flex items-center gap-2">
                <div class="flex-shrink-0">
                    <svg width="16" height="16" viewBox="0 0 16 16">
                        <circle
                            cx="8" cy="8" r="6.5"
                            fill="none"
                            stroke="var(--text-secondary)"
                            stroke-width="2"
                            opacity="0.5"
                        />
                    </svg>
                </div>
                <input
                    type="text"
                    data-testid="inline-edit-input"
                    node_ref=input_ref
                    class="flex-1 pt-0.5 bg-transparent border-none \
                           text-sm text-text-primary \
                           focus:outline-none focus-visible:outline-none \
                           no-focus-ring"
                    prop:value=move || value.get()
                    on:input=move |ev| {
                        set_value.set(event_target_value(&ev));
                    }
                    on:keydown=move |ev| {
                        ev.stop_propagation();
                        match ev.key().as_str() {
                            "Enter" => {
                                ev.prevent_default();
                                let title =
                                    value.get_untracked().trim().to_string();
                                ctrl.save_edit(title);
                            }
                            "Escape" => {
                                ev.prevent_default();
                                ctrl.cancel_edit();
                            }
                            _ => {}
                        }
                    }
                    on:blur=move |_| {
                        ctrl.cancel_edit();
                    }
                />
            </div>
        </div>
    }
}

/// Borderless inline textarea for creating a new task.
/// Supports multiline: first line becomes title, remaining lines become body.
/// Ctrl+Enter inserts a newline; plain Enter submits.
#[component]
fn InlineCreateInput(
    depth: Memo<u8>,
    value: RwSignal<String>,
    ctrl: TraversableTaskListController,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Textarea>::new();
    // Snapshot the mode that created this input instance.
    // On blur, only close if the mode hasn't changed (genuine click-away).
    // If the mode already transitioned (chaining after create), skip close.
    let created_mode = ctrl.inline_mode.get_untracked();

    let auto_resize = move || {
        if let Some(el) = input_ref.get_untracked() {
            if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                let _ = html_el.style().set_property("height", "auto");
                let scroll_h = html_el.scroll_height();
                let _ = html_el
                    .style()
                    .set_property("height", &format!("{scroll_h}px"));
            }
        }
    };

    Effect::new(move || {
        // Re-run whenever depth changes (indent/outdent) to keep focus.
        let _ = depth.get();
        if let Some(el) = input_ref.get() {
            let _ = el.focus();
        }
    });

    view! {
        <div
            style=move || {
                format!("padding-left: {}rem", depth.get() as f32 * 1.5)
            }
            class="pr-4 py-1"
        >
            <div class="flex items-start gap-2">
                <div class="flex-shrink-0 pt-1">
                    <svg width="16" height="16" viewBox="0 0 16 16">
                        <circle
                            cx="8" cy="8" r="6.5"
                            fill="none"
                            stroke="var(--text-secondary)"
                            stroke-width="2"
                            opacity="0.5"
                        />
                    </svg>
                </div>
                <div class="relative flex-1">
                    <MirrorOverlay value=Signal::derive(move || value.get()) />
                    <textarea
                        data-testid="inline-create-input"
                        node_ref=input_ref
                        class="w-full pt-0.5 bg-transparent border-none \
                               text-sm textarea-mirror \
                               focus:outline-none focus-visible:outline-none \
                               no-focus-ring resize-none overflow-hidden"
                        placeholder="Task title..."
                        rows=1
                        prop:value=move || value.get()
                        on:input=move |ev| {
                            value.set(event_target_value(&ev));
                            auto_resize();
                        }
                        on:keydown=move |ev| {
                            ev.stop_propagation();
                            if ev.key() == "Enter" {
                                if ev.ctrl_key() || ev.meta_key() {
                                    // Ctrl/Cmd+Enter: insert line break
                                    ev.prevent_default();
                                    if let Some(el) = input_ref.get_untracked() {
                                        let ta: &web_sys::HtmlTextAreaElement = &el;
                                        crate::libs::insert_newline_at_cursor(ta);
                                    }
                                    return;
                                }
                                // Plain Enter: submit
                                ev.prevent_default();
                                ctrl.create_task();
                            } else if ev.key() == "Escape" {
                                ev.prevent_default();
                                ctrl.close_inline();
                            }
                        }
                        on:blur=move |_| {
                            if ctrl.inline_mode.get_untracked() == created_mode {
                                ctrl.close_inline();
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
