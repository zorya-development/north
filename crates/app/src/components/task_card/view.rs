use std::sync::Arc;

use leptos::prelude::*;
use north_domain::TagInfo;
use wasm_bindgen::JsCast;

use crate::components::date_picker::DateTimePicker;
use crate::components::drag_drop::{DragDropContext, DropZone};
use crate::components::project_picker::ProjectPicker;
use crate::components::tag_picker::TagPicker;
use crate::components::task_detail_modal::TaskDetailContext;
use crate::components::task_form::EditTaskForm;
use crate::components::task_meta::TaskMeta;
use crate::server_fns::tasks::get_subtasks;
use north_ui::{Checkbox, DropdownItem, DropdownMenu, Icon, IconKind, MarkdownView};

#[allow(unused_variables)]
#[component]
pub fn TaskCardView(
    task_id: i64,
    title: String,
    body: Option<String>,
    sort_key: String,
    parent_id: Option<i64>,
    project_id: Option<i64>,
    project_title: Option<String>,
    due_date: Option<chrono::NaiveDate>,
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    reviewed_at: Option<chrono::NaiveDate>,
    tags: Vec<TagInfo>,
    is_completed: ReadSignal<bool>,
    editing: ReadSignal<bool>,
    set_editing: WriteSignal<bool>,
    menu_open: ReadSignal<bool>,
    set_menu_open: WriteSignal<bool>,
    on_toggle: Callback<()>,
    on_delete: Arc<dyn Fn() + Send + Sync>,
    on_save: Arc<dyn Fn(String, Option<String>) + Send + Sync>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    on_set_tags: Callback<(i64, Vec<String>)>,
    on_review: Callback<i64>,
    on_toggle_complete: Callback<(i64, bool)>,
    on_delete_task: Callback<i64>,
    on_update_task: Callback<(i64, String, Option<String>)>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = true)] show_project: bool,
    #[prop(default = 0)] subtask_count: i64,
    #[prop(default = 0)] completed_subtask_count: i64,
    #[prop(default = false)] draggable: bool,
    #[prop(default = 0)] depth: u8,
) -> impl IntoView {
    let _ = parent_id;
    let detail_ctx = use_context::<TaskDetailContext>();
    let drag_ctx = use_context::<DragDropContext>();
    let edit_title = title.clone();
    let edit_body = body.clone();
    let (subtasks_expanded, set_subtasks_expanded) = signal(false);
    let has_subtasks = subtask_count > 0;
    let (hovered, set_hovered) = signal(false);

    let indent_class = match depth {
        1 => "pl-6",
        2 => "pl-12",
        _ => "",
    };

    view! {
        <Show
            when=move || editing.get()
            fallback={
                let title = title.clone();
                let body = body.clone();
                let project_title = project_title.clone();
                let tags = tags.clone();
                let on_delete = on_delete.clone();
                let sort_key = sort_key.clone();
                move || {
                    let title = title.clone();
                    let body = body.clone();
                    let project_title = project_title.clone();
                    let meta_tags = tags.clone();
                    let on_delete = on_delete.clone();
                    let sort_key = sort_key.clone();
                    view! {
                        <div
                            on:mouseenter=move |_| set_hovered.set(true)
                            on:mouseleave=move |_| set_hovered.set(false)
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
                                            classes.push_str(
                                                " dnd-drop-above"
                                            );
                                        }
                                        Some((id, DropZone::Below)) if id == task_id => {
                                            classes.push_str(
                                                " dnd-drop-below"
                                            );
                                        }
                                        Some((id, DropZone::Nest)) if id == task_id => {
                                            classes.push_str(
                                                " dnd-drop-nest"
                                            );
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
                                            &format!(
                                                "{}|{}",
                                                task_id, sort_key,
                                            ),
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
                                // Compute drop zone from mouse Y
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
                                    ctx.drop_target
                                        .set(Some((task_id, zone)));
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
                            on:click=move |_| {
                                if let Some(ctx) = detail_ctx {
                                    ctx.open_task_id.set(Some(task_id));
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
                                            on:click=move |ev| {
                                                ev.stop_propagation()
                                            }
                                            on:mousedown=move |ev| {
                                                ev.stop_propagation()
                                            }
                                        >
                                            <Icon
                                                kind=IconKind::DragHandle
                                                class="w-4 h-4 \
                                                       text-text-tertiary"
                                            />
                                        </span>
                                    })
                                } else {
                                    None
                                }}
                                <div class="flex items-center" on:click=move |ev| ev.stop_propagation()>
                                    <Checkbox
                                        checked=is_completed
                                        on_toggle=on_toggle
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
                                                on_review.run(task_id);
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
                                <div
                                    class=move || format!(
                                        "{} transition-opacity flex \
                                         items-center",
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
                                        class="p-1 rounded \
                                               hover:bg-bg-input \
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
                                        on_set_start_at=on_set_start_at
                                        on_clear_start_at=on_clear_start_at
                                        icon_only=true
                                    />
                                    <ProjectPicker
                                        task_id=task_id
                                        project_id=project_id
                                        project_title=project_title.clone()
                                        on_set_project=on_set_project
                                        on_clear_project=on_clear_project
                                        icon_only=true
                                    />
                                    <TagPicker
                                        task_id=task_id
                                        tags=meta_tags.clone()
                                        on_set_tags=on_set_tags
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
                                            on_click={
                                                let on_delete =
                                                    on_delete.clone();
                                                move || {
                                                    set_menu_open
                                                        .set(false);
                                                    on_delete();
                                                }
                                            }
                                            danger=true
                                        />
                                    </DropdownMenu>
                                </div>
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
                                subtask_count=subtask_count
                                completed_subtask_count=completed_subtask_count
                                on_toggle_subtasks=Callback::new(move |()| {
                                    set_subtasks_expanded
                                        .update(|v| *v = !*v);
                                })
                            />
                            {if has_subtasks {
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
                                                on_toggle_complete=on_toggle_complete
                                                on_delete=on_delete_task
                                                on_update=on_update_task
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
                let on_save = on_save.clone();
                move || {
                    let on_save = on_save.clone();
                    view! {
                        <div class="px-3 py-2">
                            <EditTaskForm
                                title=edit_title.clone()
                                body=edit_body.clone()
                                on_save=move |t, b| on_save(t, b)
                                on_cancel=move || set_editing.set(false)
                            />
                        </div>
                    }
                }
            }
        </Show>
    }
}

#[component]
fn InlineSubtaskList(
    parent_id: i64,
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
    let subtasks = Resource::new(move || parent_id, get_subtasks);

    view! {
        <Suspense fallback=|| ()>
            {move || {
                Suspend::new(async move {
                    match subtasks.await {
                        Ok(tasks) => {
                            view! {
                                <div class="ml-4">
                                    {tasks
                                        .into_iter()
                                        .map(|task| {
                                            view! {
                                                <super::TaskCard
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
