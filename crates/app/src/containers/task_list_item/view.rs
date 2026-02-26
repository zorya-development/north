use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::atoms::{TextColor, TextVariant};
use crate::components::date_picker::DateTimePicker;
use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::components::rich_title::RichTitle;
use crate::containers::project_picker::ProjectPicker;
use crate::containers::tag_picker::TagPicker;
use crate::containers::task_checkbox::TaskCheckbox;
use crate::containers::task_meta::TaskMeta;
use north_stores::{use_app_store, TaskModel};
use north_ui::{DropdownItem, DropdownMenu, Icon, IconKind};

#[component]
pub fn TaskListItemView(
    task: Memo<Option<TaskModel>>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = false)] show_inline_project: bool,
    #[prop(default = true)] show_inline_tags: bool,
    #[prop(default = false)] draggable: bool,
    on_delete: Callback<()>,
    on_review: Callback<()>,
    on_set_start_at: Callback<String>,
    on_clear_start_at: Callback<()>,
    on_set_project: Callback<i64>,
    on_clear_project: Callback<()>,
    on_set_tags: Callback<Vec<String>>,
) -> impl IntoView {
    let _ = show_project; // Used by ItemConfig for future TaskMeta project display
    let app_store = use_app_store();
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
            let recurrence = t.recurrence.clone();

            view! {
                <div
                    on:mouseenter=move |_| set_hovered.set(true)
                    on:mouseleave=move |_| set_hovered.set(false)
                    class=move || {
                        let mut classes = "relative pr-4 \
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
                            // Defer the clear so that dragover on the
                            // next element fires first, preventing a
                            // one-frame blink when crossing adjacent
                            // task boundaries.
                            request_animation_frame(move || {
                                if ctx.drop_target.get_untracked()
                                    .map(|(id, _)| id)
                                    == Some(task_id)
                                {
                                    ctx.drop_target.set(None);
                                }
                            });
                        }
                    }
                >
                    {if draggable {
                        Some(view! {
                            <span
                                class=move || format!(
                                    "dnd-handle absolute left-0 \
                                     top-1/2 -translate-x-full \
                                     -translate-y-1/2 \
                                     {} cursor-grab \
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
                    <div class="flex items-start gap-2">
                        <div
                            class="flex items-center mt-1"
                            on:click=move |ev| ev.stop_propagation()
                        >
                            <TaskCheckbox task_id=task_id/>
                        </div>
                        {
                        let inline_tags = tags.clone();
                        move || {
                            let completed = is_completed.get();
                            let t = title.clone();

                            // Project prefix: @ProjectName or @Inbox
                            let project_prefix = if show_inline_project {
                                if let Some(pid) = project_id {
                                    let projects = app_store.projects.get();
                                    projects.iter().find(|p| p.id == pid).map(|project| {
                                        let color = project.color.clone();
                                        let title = project.title.clone();
                                        let href = format!("/projects/{pid}");
                                        view! {
                                            <span
                                                class="text-sm font-medium mr-1"
                                                style=format!("color: {color}")
                                            >
                                                "@"
                                                <a
                                                    href=href
                                                    class="hover:underline"
                                                    on:click=move |ev: leptos::ev::MouseEvent| {
                                                        ev.stop_propagation();
                                                    }
                                                >
                                                    {title}
                                                </a>
                                            </span>
                                        }.into_any()
                                    })
                                } else {
                                    Some(view! {
                                        <span class="text-sm font-medium mr-1 text-text-tertiary">
                                            "@"
                                            <a
                                                href="/inbox"
                                                class="hover:underline"
                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                    ev.stop_propagation();
                                                }
                                            >
                                                "Inbox"
                                            </a>
                                        </span>
                                    }.into_any())
                                }
                            } else {
                                None
                            };

                            // Tag suffix: #tag1 #tag2
                            let tag_suffix = if show_inline_tags && !inline_tags.is_empty() {
                                let tag_views = inline_tags.iter().map(|tag| {
                                    let query = format!("tags=\"{}\"", tag.name);
                                    let encoded = urlencoding::encode(&query).into_owned();
                                    let href = format!("/filters/new?q={encoded}");
                                    let name = tag.name.clone();
                                    view! {
                                        <span class="text-text-secondary text-sm ml-1.5">
                                            "#"
                                            <a
                                                href=href
                                                class="hover:underline"
                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                    ev.stop_propagation();
                                                }
                                            >
                                                {name}
                                            </a>
                                        </span>
                                    }
                                }).collect::<Vec<_>>();
                                Some(tag_views)
                            } else {
                                None
                            };

                            let color = if completed {
                                TextColor::Tertiary
                            } else {
                                TextColor::Primary
                            };

                            view! {
                                <span class="flex-1 pt-0.5 flex items-baseline flex-wrap">
                                    {project_prefix}
                                    <RichTitle
                                        title=t
                                        variant=TextVariant::BodyMd
                                        color=color
                                        line_through=completed
                                    />
                                    {tag_suffix}
                                </span>
                            }
                        }
                        }
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
                        show_tags=!show_inline_tags
                        on_review=on_review
                        recurrence=recurrence
                        class="pl-6"
                    />
                </div>
            }
            .into_any()
        }}
    }
}
