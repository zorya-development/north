use leptos::prelude::*;
use north_ui::Spinner;
use wasm_bindgen::JsCast;

use super::controller::TraversableTaskListController;
use super::tree::*;
use crate::atoms::{Text, TextColor, TextTag, TextVariant};
use crate::containers::task_list_item::TaskListItem;

#[component]
pub fn TraversableTaskListView(
    ctrl: TraversableTaskListController,
    #[prop(default = true)] show_project: bool,
    #[prop(default = false)] draggable: bool,
    #[prop(default = "No tasks.")] empty_message: &'static str,
    is_loaded: Signal<bool>,
    #[prop(default = false)] scoped: bool,
) -> impl IntoView {
    let flat_nodes = ctrl.flat_nodes;
    let cursor_task_id = ctrl.cursor_task_id;
    let inline_mode = ctrl.inline_mode;
    let create_input_value = ctrl.create_input_value;
    let show_review = ctrl.show_review;
    let container_ref = NodeRef::<leptos::html::Div>::new();

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
            class=if scoped { "focus:outline-none" } else { "" }
            on:keydown=on_keydown
            on:click=on_container_click
            on:focus=on_focus
        >
            {move || {
                if !is_loaded.get() {
                    return view! { <Spinner/> }.into_any();
                }
                let nodes = flat_nodes.get();
                if nodes.is_empty() {
                    return view! {
                        <Text
                            variant=TextVariant::BodyMd
                            color=TextColor::Secondary
                            tag=TextTag::P
                            class="py-8 text-center"
                        >
                            {empty_message}
                        </Text>
                    }
                    .into_any();
                }
                view! {
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
                                            show_project=show_project
                                            show_review=show_review
                                            draggable=draggable
                                            hide_subtasks=true
                                            hide_body=true
                                            depth=0
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
                }
                .into_any()
            }}
        </div>
    }
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
            class="px-4 py-1 trash-polka-focus"
        >
            <div class="flex items-center gap-2">
                <div class="w-5 h-5 shrink-0"/>
                <input
                    type="text"
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

/// Borderless inline input for creating a new task.
#[component]
fn InlineCreateInput(
    depth: Memo<u8>,
    value: RwSignal<String>,
    ctrl: TraversableTaskListController,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();

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
            class="px-4 py-1"
        >
            <div class="flex items-center gap-2">
                <div class="w-5 h-5 shrink-0 text-text-tertiary">
                    <svg
                        viewBox="0 0 20 20"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.5"
                        class="w-5 h-5"
                    >
                        <line x1="10" y1="4" x2="10" y2="16"/>
                        <line x1="4" y1="10" x2="16" y2="10"/>
                    </svg>
                </div>
                <input
                    type="text"
                    node_ref=input_ref
                    class="flex-1 pt-0.5 bg-transparent border-none \
                           text-sm text-text-primary \
                           placeholder-text-tertiary \
                           focus:outline-none focus-visible:outline-none \
                           no-focus-ring"
                    placeholder="Task title..."
                    prop:value=move || value.get()
                    on:input=move |ev| {
                        value.set(event_target_value(&ev));
                    }
                    on:keydown=move |ev| {
                        ev.stop_propagation();
                        match ev.key().as_str() {
                            "Enter" => {
                                ev.prevent_default();
                                ctrl.create_task();
                            }
                            "Escape" => {
                                ev.prevent_default();
                                ctrl.close_inline();
                            }
                            _ => {}
                        }
                    }
                    on:blur=move |_| {
                        ctrl.close_inline();
                    }
                />
            </div>
        </div>
    }
}
