use std::sync::Arc;

use leptos::prelude::*;

use crate::components::completion_toggle::CompletionToggle;
use crate::components::dropdown::{DropdownItem, DropdownMenu};
use crate::components::icons::{Icon, IconKind};
use crate::components::markdown::MarkdownView;
use crate::components::task_form::EditTaskForm;
use crate::components::task_meta::TaskMeta;

#[component]
pub fn TaskCardView(
    task_id: i64,
    title: String,
    body: Option<String>,
    project_id: Option<i64>,
    project_title: Option<String>,
    due_date: Option<chrono::NaiveDate>,
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    tags: Vec<String>,
    is_completed: ReadSignal<bool>,
    editing: ReadSignal<bool>,
    set_editing: WriteSignal<bool>,
    menu_open: ReadSignal<bool>,
    set_menu_open: WriteSignal<bool>,
    on_toggle: Arc<dyn Fn() + Send + Sync>,
    on_delete: Arc<dyn Fn() + Send + Sync>,
    on_save: Arc<dyn Fn(String, Option<String>) + Send + Sync>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
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
                let on_toggle = on_toggle.clone();
                let on_delete = on_delete.clone();
                move || {
                    let title = title.clone();
                    let body = body.clone();
                    let project_title = project_title.clone();
                    let tags = tags.clone();
                    let on_toggle = on_toggle.clone();
                    let on_delete = on_delete.clone();
                    view! {
                        <div class="group border-b border-border px-3 py-2 \
                                    hover:bg-white/10 transition-colors">
                            <div class="flex items-center gap-2">
                                <CompletionToggle
                                    is_completed=is_completed
                                    on_toggle=on_toggle.clone()
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
                                is_completed=is_completed
                                tags=tags
                                on_set_start_at=on_set_start_at
                                on_clear_start_at=on_clear_start_at
                                on_set_project=on_set_project
                                on_clear_project=on_clear_project
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
