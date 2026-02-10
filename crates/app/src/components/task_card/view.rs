use std::sync::Arc;

use leptos::prelude::*;
use north_domain::TagInfo;

use crate::components::task_form::EditTaskForm;
use crate::components::task_meta::TaskMeta;
use north_ui::{Checkbox, DropdownItem, DropdownMenu, Icon, IconKind, MarkdownView};

#[component]
pub fn TaskCardView(
    task_id: i64,
    title: String,
    body: Option<String>,
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
    #[prop(default = false)] show_review: bool,
) -> impl IntoView {
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
                let on_delete = on_delete.clone();
                move || {
                    let title = title.clone();
                    let body = body.clone();
                    let project_title = project_title.clone();
                    let tags = tags.clone();
                    let on_delete = on_delete.clone();
                    view! {
                        <div class="group border-b border-border px-3 py-2 \
                                    hover:bg-white/10 transition-colors">
                            <div class="flex items-center gap-2">
                                <Checkbox
                                    checked=is_completed
                                    on_toggle=on_toggle
                                    checked_label="Mark task incomplete"
                                    unchecked_label="Complete task"
                                />
                                <span class=move || {
                                    if is_completed.get() {
                                        "flex-1 text-sm text-text-tertiary \
                                         line-through"
                                    } else {
                                        "flex-1 text-sm text-text-primary"
                                    }
                                }>
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
                                <div class="opacity-0 group-hover:opacity-100 \
                                            transition-opacity">
                                    <DropdownMenu
                                        open=menu_open
                                        set_open=set_menu_open
                                        trigger=Box::new(move || {
                                            view! {
                                                <button
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                        set_menu_open
                                                            .update(|o| *o = !*o);
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
                                task_id=task_id
                                start_at=start_at
                                project_id=project_id
                                project_title=project_title
                                due_date=due_date
                                tags=tags
                                on_set_start_at=on_set_start_at
                                on_clear_start_at=on_clear_start_at
                                on_set_project=on_set_project
                                on_clear_project=on_clear_project
                                on_set_tags=on_set_tags
                                reviewed_at=reviewed_at
                                show_review=show_review
                            />
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
