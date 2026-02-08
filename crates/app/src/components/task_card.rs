use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::dropdown::{DropdownItem, DropdownMenu};
use crate::components::markdown::MarkdownView;
use crate::components::task_form::EditTaskForm;

#[component]
pub fn TaskCard<D, U>(
    task: TaskWithMeta,
    on_delete: D,
    on_update: U,
) -> impl IntoView
where
    D: Fn(i64) + Send + Sync + Clone + 'static,
    U: Fn(i64, String, Option<String>) + Send + Sync + Clone + 'static,
{
    let task_id = task.task.id;
    let title = task.task.title.clone();
    let body = task.task.body.clone();
    let project_title = task.project_title.clone();
    let due_date = task.task.due_date;
    let tags = task.tags.clone();

    let (editing, set_editing) = signal(false);
    let (menu_open, set_menu_open) = signal(false);

    let edit_title = title.clone();
    let edit_body = body.clone();

    let on_edit = {
        move || {
            set_menu_open.set(false);
            set_editing.set(true);
        }
    };

    let on_delete_click = {
        let on_delete = on_delete.clone();
        move || {
            set_menu_open.set(false);
            on_delete(task_id);
        }
    };

    let on_save = {
        let on_update = on_update.clone();
        move |new_title: String, new_body: Option<String>| {
            set_editing.set(false);
            on_update(task_id, new_title, new_body);
        }
    };

    let on_cancel_edit = move || {
        set_editing.set(false);
    };

    view! {
        <Show
            when=move || editing.get()
            fallback={
                let title = title.clone();
                let body = body.clone();
                let project_title = project_title.clone();
                let tags = tags.clone();
                move || {
                    let title = title.clone();
                    let body = body.clone();
                    let project_title = project_title.clone();
                    let tags = tags.clone();
                    view! {
                        <div class="group border-b border-border px-3 py-2 \
                                    hover:bg-bg-tertiary transition-colors">
                            <div class="flex items-center gap-2">
                                <button
                                    class="w-4 h-4 rounded-full border-2 \
                                           border-text-secondary \
                                           hover:border-accent \
                                           hover:bg-accent \
                                           transition-colors flex-shrink-0"
                                    aria-label="Complete task"
                                />
                                <span class="flex-1 text-sm text-text-primary">
                                    {title}
                                </span>
                                <div class="opacity-0 group-hover:opacity-100 \
                                            transition-opacity">
                                    <DropdownMenu
                                        open=menu_open
                                        set_open=set_menu_open
                                        trigger=move || {
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
                                                    <svg
                                                        xmlns="http://www.w3.org/2000/svg"
                                                        width="16"
                                                        height="16"
                                                        viewBox="0 0 24 24"
                                                        fill="currentColor"
                                                    >
                                                        <circle cx="12" cy="5" r="2"/>
                                                        <circle cx="12" cy="12" r="2"/>
                                                        <circle cx="12" cy="19" r="2"/>
                                                    </svg>
                                                </button>
                                            }
                                        }
                                    >
                                        <DropdownItem
                                            label="Edit"
                                            on_click=on_edit
                                        />
                                        <DropdownItem
                                            label="Delete"
                                            on_click=on_delete_click
                                            danger=true
                                        />
                                    </DropdownMenu>
                                </div>
                            </div>

                            {body
                                .map(|b| {
                                    view! {
                                        <div class="mt-1 ml-6">
                                            <MarkdownView content=b/>
                                        </div>
                                    }
                                })}

                            {(!project_title.is_none()
                                || due_date.is_some()
                                || !tags.is_empty())
                            .then(move || {
                                let project_title = project_title.clone();
                                let tags = tags.clone();
                                view! {
                                    <div class="mt-0.5 ml-6 flex items-center \
                                                gap-2 text-xs text-text-tertiary">
                                        {project_title
                                            .map(|p| {
                                                view! {
                                                    <span class="text-text-secondary">
                                                        {p}
                                                    </span>
                                                }
                                            })}
                                        {due_date
                                            .map(|d| {
                                                view! {
                                                    <span>{format!("Due {d}")}</span>
                                                }
                                            })}
                                        {tags
                                            .into_iter()
                                            .map(|tag| {
                                                view! {
                                                    <span class="bg-bg-tertiary \
                                                                 text-text-secondary \
                                                                 text-xs px-2 py-0.5 \
                                                                 rounded-full">
                                                        {tag}
                                                    </span>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </div>
                                }
                            })}
                        </div>
                    }
                }
            }
        >
            <div class="px-3 py-2">
                <EditTaskForm
                    title=edit_title
                    body=edit_body
                    on_save=on_save
                    on_cancel=on_cancel_edit
                />
            </div>
        </Show>
    }
}
