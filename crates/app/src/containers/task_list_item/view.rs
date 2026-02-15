use leptos::prelude::*;
use wasm_bindgen::JsCast;

use super::components::InlineSubtaskList;
use super::controller::TaskListItemController;
use crate::components::date_picker::DateTimePicker;
use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::components::project_picker::ProjectPicker;
use crate::components::tag_picker::TagPicker;
use crate::components::task_form::EditTaskForm;
use crate::components::task_meta::TaskMeta;
use north_ui::{Checkbox, DropdownItem, DropdownMenu, Icon, IconKind, MarkdownView};

#[component]
pub fn TaskListItemView(
    ctrl: TaskListItemController,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = false)] draggable: bool,
    #[prop(default = false)] compact: bool,
    #[prop(default = 0)] depth: u8,
    on_click: Option<Callback<i64>>,
) -> impl IntoView {
    let task = ctrl.task;
    let (editing, set_editing) = ctrl.editing;
    let (menu_open, set_menu_open) = ctrl.menu_open;
    let drag_ctx = use_context::<DragDropContext>();
    let (subtasks_expanded, _set_subtasks_expanded) = signal(true);
    let (hovered, set_hovered) = signal(false);

    let indent_class = match depth {
        1 => "pl-6",
        2 => "pl-12",
        _ => "",
    };

    let is_completed = Memo::new(move |_| {
        task.get()
            .map(|t| t.completed_at.is_some())
            .unwrap_or(false)
    });

    view! {
        {move || {
            let Some(t) = task.get() else {
                return view! { <div/> }.into_any();
            };

            let task_id = t.id;
            let title = t.title.clone();
            let body = t.body.clone();
            let sort_key = t.sort_key.clone();
            let project_id = t.project_id;
            let project_title = t.project_title.clone();
            let due_date = t.due_date;
            let start_at = t.start_at;
            let reviewed_at = t.reviewed_at;
            let tags = t.tags.clone();
            let subtask_count = t.subtask_count;
            let _completed_subtask_count = t.completed_subtask_count;
            let sequential_limit = t.sequential_limit;
            let has_subtasks = subtask_count > 0;

            let edit_title = title.clone();
            let edit_body = body.clone();

            view! {
                <Show
                    when=move || editing.get()
                    fallback={
                        let title = title.clone();
                        let body = body.clone();
                        let project_title = project_title.clone();
                        let tags = tags.clone();
                        let sort_key = sort_key.clone();
                        move || {
                            let title = title.clone();
                            let body = body.clone();
                            let project_title = project_title.clone();
                            let meta_tags = tags.clone();
                            let sort_key = sort_key.clone();
                            view! {
                                <div
                                    on:mouseenter=move |_| set_hovered.set(true)
                                    on:mouseleave=move |_| set_hovered.set(false)
                                    on:click=move |_| {
                                        if let Some(cb) = on_click {
                                            cb.run(task_id);
                                        }
                                    }
                                    class=move || {
                                        let mut classes = format!(
                                            "px-4 \
                                             hover:bg-hover-overlay transition-colors \
                                             cursor-pointer {indent_class}"
                                        );
                                        if let Some(ctx) = drag_ctx {
                                            if ctx.dragging_task_id.get() == Some(task_id) {
                                                classes.push_str(" opacity-30");
                                            }
                                            match ctx.drop_target.get() {
                                                Some((id, DropZone::Above)) if id == task_id => {
                                                    classes.push_str(" dnd-drop-above");
                                                }
                                                Some((id, DropZone::Below)) if id == task_id => {
                                                    classes.push_str(" dnd-drop-below");
                                                }
                                                Some((id, DropZone::Nest)) if id == task_id => {
                                                    classes.push_str(" dnd-drop-nest");
                                                }
                                                _ => {}
                                            }
                                        }
                                        classes
                                    }
                                    draggable=move || {
                                        if draggable { "true" } else { "false" }
                                    }
                                    on:dragstart={
                                        let sort_key = sort_key.clone();
                                        move |ev: web_sys::DragEvent| {
                                            if !draggable { return; }
                                            if let Some(ctx) = drag_ctx {
                                                ctx.dragging_task_id.set(Some(task_id));
                                            }
                                            if let Some(dt) = ev.data_transfer() {
                                                let _ = dt.set_data(
                                                    "text/plain",
                                                    &format!("{}|{}", task_id, sort_key),
                                                );
                                                dt.set_effect_allowed("move");
                                            }
                                        }
                                    }
                                    on:dragend=move |_: web_sys::DragEvent| {
                                        if let Some(ctx) = drag_ctx {
                                            ctx.dragging_task_id.set(None);
                                            ctx.drop_target.set(None);
                                        }
                                    }
                                    on:dragover=move |ev: web_sys::DragEvent| {
                                        if drag_ctx.is_none() { return; }
                                        let ctx = drag_ctx.unwrap();
                                        if ctx.dragging_task_id.get_untracked()
                                            == Some(task_id)
                                        {
                                            return;
                                        }
                                        ev.prevent_default();
                                        if let Some(target) = ev
                                            .current_target()
                                            .and_then(|t| {
                                                t.dyn_into::<web_sys::HtmlElement>().ok()
                                            })
                                        {
                                            let rect = target.get_bounding_client_rect();
                                            let y = ev.client_y() as f64 - rect.top();
                                            let h = rect.height();
                                            let zone = if y < h * 0.25 {
                                                DropZone::Above
                                            } else if y > h * 0.75 {
                                                DropZone::Below
                                            } else {
                                                DropZone::Nest
                                            };
                                            ctx.drop_target.set(Some((task_id, zone)));
                                        }
                                    }
                                    on:dragleave=move |_: web_sys::DragEvent| {
                                        if let Some(ctx) = drag_ctx {
                                            if ctx.drop_target.get_untracked()
                                                .map(|(id, _)| id) == Some(task_id)
                                            {
                                                ctx.drop_target.set(None);
                                            }
                                        }
                                    }
                                >
                                    <div class="flex items-center gap-2">
                                        {if draggable {
                                            Some(view! {
                                                <span
                                                    class=move || format!(
                                                        "dnd-handle {} cursor-grab \
                                                         active:cursor-grabbing \
                                                         transition-opacity",
                                                        if hovered.get() {
                                                            "opacity-60"
                                                        } else {
                                                            "opacity-0"
                                                        },
                                                    )
                                                    on:click=move |ev| ev.stop_propagation()
                                                    on:mousedown=move |ev| ev.stop_propagation()
                                                >
                                                    <Icon
                                                        kind=IconKind::DragHandle
                                                        class="w-4 h-4 text-text-tertiary"
                                                    />
                                                </span>
                                            })
                                        } else {
                                            None
                                        }}
                                        <div
                                            class="flex items-center"
                                            on:click=move |ev| ev.stop_propagation()
                                        >
                                            <Checkbox
                                                checked=is_completed
                                                on_toggle=Callback::new(move |()| {
                                                    ctrl.toggle_complete()
                                                })
                                                checked_label="Mark task incomplete"
                                                unchecked_label="Complete task"
                                            />
                                        </div>
                                        <span
                                            class=move || {
                                                if is_completed.get() {
                                                    "flex-1 text-sm text-text-tertiary \
                                                     line-through"
                                                } else {
                                                    "flex-1 text-sm text-text-primary"
                                                }
                                            }
                                        >
                                            {title}
                                        </span>
                                        {if show_review {
                                            Some(view! {
                                                <button
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                        ctrl.review();
                                                    }
                                                    class="px-2 py-0.5 text-xs rounded \
                                                           border border-border \
                                                           text-text-secondary \
                                                           hover:bg-bg-tertiary \
                                                           hover:text-text-primary \
                                                           transition-colors"
                                                >
                                                    "Reviewed"
                                                </button>
                                            })
                                        } else {
                                            None
                                        }}
                                        {if !compact {
                                            Some(view! {
                                                <div
                                                    class=move || format!(
                                                        "{} transition-opacity \
                                                         flex items-center",
                                                        if hovered.get() {
                                                            "opacity-100"
                                                        } else {
                                                            "opacity-0"
                                                        },
                                                    )
                                                    on:click=move |ev| ev.stop_propagation()
                                                >
                                                    <button
                                                        on:click=move |ev| {
                                                            ev.stop_propagation();
                                                            set_editing.set(true);
                                                        }
                                                        class="p-1 rounded hover:bg-bg-input \
                                                               text-text-tertiary \
                                                               hover:text-text-secondary \
                                                               transition-colors"
                                                        aria-label="Edit task"
                                                    >
                                                        <Icon
                                                            kind=IconKind::Edit
                                                            class="w-4 h-4"
                                                        />
                                                    </button>
                                                    <DateTimePicker
                                                        task_id=task_id
                                                        start_at=start_at
                                                        on_set_start_at=Callback::new(
                                                            move |(_, sa)| {
                                                                ctrl.set_start_at(sa)
                                                            },
                                                        )
                                                        on_clear_start_at=Callback::new(
                                                            move |_| ctrl.clear_start_at(),
                                                        )
                                                        icon_only=true
                                                    />
                                                    <ProjectPicker
                                                        task_id=task_id
                                                        project_id=project_id
                                                        project_title=project_title.clone()
                                                        on_set_project=Callback::new(
                                                            move |(_, pid)| {
                                                                ctrl.set_project(pid)
                                                            },
                                                        )
                                                        on_clear_project=Callback::new(
                                                            move |_| ctrl.clear_project(),
                                                        )
                                                        icon_only=true
                                                    />
                                                    <TagPicker
                                                        task_id=task_id
                                                        tags=meta_tags.clone()
                                                        on_set_tags=Callback::new(
                                                            move |(_, tags)| {
                                                                ctrl.set_tags(tags)
                                                            },
                                                        )
                                                        icon_only=true
                                                    />
                                                    <DropdownMenu
                                                        open=menu_open
                                                        set_open=set_menu_open
                                                        trigger=Box::new(move || {
                                                            view! {
                                                                <button
                                                                    on:click=move |ev| {
                                                                        ev.stop_propagation();
                                                                        set_menu_open
                                                                            .update(|o| {
                                                                                *o = !*o
                                                                            });
                                                                    }
                                                                    class="p-1 rounded \
                                                                           hover:bg-bg-input \
                                                                           text-text-tertiary \
                                                                           hover:text-text-secondary \
                                                                           transition-colors"
                                                                    aria-label="Task actions"
                                                                >
                                                                    <Icon
                                                                        kind=IconKind::KebabMenu
                                                                        class="w-4 h-4"
                                                                    />
                                                                </button>
                                                            }.into_any()
                                                        })
                                                    >
                                                        <DropdownItem
                                                            label="Edit"
                                                            on_click=move || {
                                                                set_menu_open.set(false);
                                                                set_editing.set(true);
                                                            }
                                                        />
                                                        <DropdownItem
                                                            label="Delete"
                                                            on_click=move || {
                                                                set_menu_open.set(false);
                                                                ctrl.delete();
                                                            }
                                                            danger=true
                                                        />
                                                    </DropdownMenu>
                                                </div>
                                            })
                                        } else {
                                            None
                                        }}
                                    </div>

                                    {body.map(|b| {
                                        view! {
                                            <div class="mt-1 ml-6">
                                                <MarkdownView content=b/>
                                            </div>
                                        }
                                    })}

                                    <TaskMeta
                                        start_at=start_at
                                        show_project=show_project
                                        project_title=project_title
                                        due_date=due_date
                                        tags=meta_tags
                                        reviewed_at=reviewed_at
                                        show_review=show_review
                                    />
                                    {if has_subtasks && !compact {
                                        Some(view! {
                                            <Show when=move || subtasks_expanded.get()>
                                                <div
                                                    on:mouseenter=move |_| {
                                                        set_hovered.set(false)
                                                    }
                                                    on:mouseleave=move |_| {
                                                        set_hovered.set(true)
                                                    }
                                                    on:click=|ev: web_sys::MouseEvent| {
                                                        ev.stop_propagation()
                                                    }
                                                    on:dragstart=|ev: web_sys::DragEvent| {
                                                        ev.stop_propagation()
                                                    }
                                                    on:dragover=|ev: web_sys::DragEvent| {
                                                        ev.stop_propagation()
                                                    }
                                                    on:dragleave=|ev: web_sys::DragEvent| {
                                                        ev.stop_propagation()
                                                    }
                                                >
                                                    <InlineSubtaskList
                                                        parent_id=task_id
                                                        sequential_limit=sequential_limit
                                                        show_project=show_project
                                                        draggable=draggable
                                                        depth={depth + 1}
                                                    />
                                                </div>
                                            </Show>
                                        })
                                    } else {
                                        None
                                    }}
                                </div>
                            }
                        }
                    }
                >
                    {
                        let edit_title = edit_title.clone();
                        let edit_body = edit_body.clone();
                        move || {
                            let edit_title = edit_title.clone();
                            let edit_body = edit_body.clone();
                            view! {
                                <div class="px-3 py-2">
                                    <EditTaskForm
                                        title=edit_title
                                        body=edit_body
                                        on_save=move |t, b| ctrl.update(t, b)
                                        on_cancel=move || set_editing.set(false)
                                    />
                                </div>
                            }
                        }
                    }
                </Show>
            }
            .into_any()
        }}
    }
}
