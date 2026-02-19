use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::atoms::{Text, TextColor, TextVariant};
use crate::components::date_picker::DateTimePicker;
use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::components::task_meta::TaskMeta;
use crate::containers::project_picker::ProjectPicker;
use crate::containers::tag_picker::TagPicker;
use crate::containers::task_checkbox::TaskCheckbox;
use north_dto::Task;
use north_ui::{DropdownItem, DropdownMenu, Icon, IconKind};

#[component]
pub fn TaskListItemView(
    task: Memo<Option<Task>>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = false)] draggable: bool,
    on_delete: Callback<()>,
    on_review: Callback<()>,
    on_set_start_at: Callback<String>,
    on_clear_start_at: Callback<()>,
    on_set_project: Callback<i64>,
    on_clear_project: Callback<()>,
    on_set_tags: Callback<Vec<String>>,
) -> impl IntoView {
    let _ = show_project; // Reserved for future use (e.g. hiding project in TaskMeta)
    let drag_ctx = use_context::<DragDropContext>();
    let (hovered, set_hovered) = signal(false);
    let (menu_open, set_menu_open) = signal(false);

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
            let sort_key = t.sort_key.clone();
            let project_id = t.project_id;
            let project_title = t.project_title.clone();
            let due_date = t.due_date;
            let start_at = t.start_at;
            let reviewed_at = t.reviewed_at;
            let tags = t.tags.clone();

            view! {
                <div
                    on:mouseenter=move |_| set_hovered.set(true)
                    on:mouseleave=move |_| set_hovered.set(false)
                    class=move || {
                        let mut classes = "px-4 \
                             hover:bg-hover-overlay transition-colors \
                             cursor-pointer"
                            .to_string();
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
                            <TaskCheckbox task_id=task_id/>
                        </div>
                        {move || {
                            let completed = is_completed.get();
                            let t = title.clone();
                            view! {
                                <Text
                                    variant=TextVariant::BodyMd
                                    color={if completed {
                                        TextColor::Tertiary
                                    } else {
                                        TextColor::Primary
                                    }}
                                    line_through=completed
                                    class="flex-1 pt-0.5"
                                >
                                    {t}
                                </Text>
                            }
                        }}
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
                            <DateTimePicker
                                task_id=task_id
                                start_at=start_at
                                on_set_start_at=Callback::new(
                                    move |(_, sa)| {
                                        on_set_start_at.run(sa)
                                    },
                                )
                                on_clear_start_at=Callback::new(
                                    move |_| on_clear_start_at.run(()),
                                )
                                icon_only=true
                            />
                            <ProjectPicker
                                task_id=task_id
                                project_id=project_id
                                project_title=project_title.clone()
                                on_set_project=Callback::new(
                                    move |(_, pid)| {
                                        on_set_project.run(pid)
                                    },
                                )
                                on_clear_project=Callback::new(
                                    move |_| on_clear_project.run(()),
                                )
                                icon_only=true
                            />
                            <TagPicker
                                task_id=task_id
                                tags=tags.clone()
                                on_set_tags=Callback::new(
                                    move |(_, tags)| {
                                        on_set_tags.run(tags)
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
                                    label="Delete"
                                    on_click=move || {
                                        set_menu_open.set(false);
                                        on_delete.run(());
                                    }
                                    danger=true
                                />
                            </DropdownMenu>
                        </div>
                    </div>

                    <TaskMeta
                        start_at=start_at
                        due_date=due_date
                        tags=tags
                        reviewed_at=reviewed_at
                        show_review=show_review
                        on_review=on_review
                        class=if draggable { "pl-12" } else { "pl-6" }
                    />
                </div>
            }
            .into_any()
        }}
    }
}
