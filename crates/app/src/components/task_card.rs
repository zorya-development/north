use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::dropdown::{DropdownItem, DropdownMenu};
use crate::components::markdown::MarkdownView;
use crate::components::task_form::EditTaskForm;

#[component]
pub fn TaskCard<C, D, U>(
    task: TaskWithMeta,
    on_toggle_complete: C,
    on_delete: D,
    on_update: U,
    #[prop(optional)] on_set_start_at: Option<Callback<(i64, String)>>,
    #[prop(optional)] on_clear_start_at: Option<Callback<i64>>,
) -> impl IntoView
where
    C: Fn(i64, bool) + Send + Sync + Clone + 'static,
    D: Fn(i64) + Send + Sync + Clone + 'static,
    U: Fn(i64, String, Option<String>) + Send + Sync + Clone + 'static,
{
    let task_id = task.task.id;
    let title = task.task.title.clone();
    let body = task.task.body.clone();
    let project_title = task.project_title.clone();
    let due_date = task.task.due_date;
    let start_at = task.task.start_at;
    let initial_completed = task.task.completed_at.is_some();
    let tags = task.tags.clone();

    let start_at_display = start_at.map(|dt| {
        dt.format("%b %-d, %-I:%M %p").to_string()
    });

    let has_start_at = start_at.is_some();

    let (is_completed, set_is_completed) = signal(initial_completed);
    let (editing, set_editing) = signal(false);
    let (menu_open, set_menu_open) = signal(false);
    let (popover_open, set_popover_open) = signal(false);

    let picked_date = RwSignal::new(String::new());
    let picked_time = RwSignal::new("09:00".to_string());

    let initial_date = start_at
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let initial_time = start_at
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|| "09:00".to_string());

    let edit_title = title.clone();
    let edit_body = body.clone();

    let on_delete = std::sync::Arc::new(on_delete);

    let on_toggle = {
        let on_toggle_complete = on_toggle_complete.clone();
        std::sync::Arc::new(move || {
            let was_completed = is_completed.get_untracked();
            set_is_completed.set(!was_completed);
            on_toggle_complete(task_id, was_completed);
        })
    };

    let on_save = {
        let on_update = on_update.clone();
        std::sync::Arc::new(
            move |new_title: String, new_body: Option<String>| {
                set_editing.set(false);
                on_update(task_id, new_title, new_body);
            },
        )
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
                let on_toggle = on_toggle.clone();
                let on_delete = on_delete.clone();
                let start_at_display = start_at_display.clone();
                let initial_date = initial_date.clone();
                let initial_time = initial_time.clone();
                move || {
                    let title = title.clone();
                    let body = body.clone();
                    let project_title = project_title.clone();
                    let tags = tags.clone();
                    let on_toggle = on_toggle.clone();
                    let on_delete = on_delete.clone();
                    let start_at_display = start_at_display.clone();
                    let initial_date = initial_date.clone();
                    let initial_time = initial_time.clone();
                    view! {
                        <div class="group border-b border-border px-3 py-2 \
                                    hover:bg-white/10 transition-colors">
                            <div class="flex items-center gap-2">
                                <button
                                    on:click={
                                        let on_toggle = on_toggle.clone();
                                        move |_| on_toggle()
                                    }
                                    class="flex-shrink-0"
                                    aria-label=move || {
                                        if is_completed.get() {
                                            "Mark task incomplete"
                                        } else {
                                            "Complete task"
                                        }
                                    }
                                >
                                    <Show
                                        when=move || is_completed.get()
                                        fallback=move || {
                                            view! {
                                                <div class="w-4 h-4 \
                                                            rounded-full \
                                                            border-2 \
                                                            border-text-secondary \
                                                            hover:border-accent \
                                                            hover:bg-accent \
                                                            transition-colors" />
                                            }
                                        }
                                    >
                                        <div class="w-4 h-4 rounded-full \
                                                    bg-text-tertiary \
                                                    hover:bg-text-secondary \
                                                    flex items-center \
                                                    justify-center \
                                                    transition-colors">
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                width="10"
                                                height="10"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="3"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                class="text-bg-primary"
                                            >
                                                <polyline
                                                    points="20 6 9 17 4 12"
                                                />
                                            </svg>
                                        </div>
                                    </Show>
                                </button>
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
                                                let on_delete = on_delete.clone();
                                                move || {
                                                    set_menu_open.set(false);
                                                    on_delete(task_id);
                                                }
                                            }
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

                            // Meta row
                            <div class="mt-0.5 ml-6 flex items-center \
                                        gap-2 text-xs text-text-tertiary">
                                // Date picker
                                <div class="relative inline-flex">
                                    {if has_start_at {
                                        let display =
                                            start_at_display.clone()
                                                .unwrap_or_default();
                                        view! {
                                            <button
                                                class="inline-flex \
                                                       items-center \
                                                       gap-1 text-accent \
                                                       hover:bg-bg-tertiary \
                                                       px-1.5 py-0.5 \
                                                       rounded \
                                                       transition-colors"
                                                on:click={
                                                    let id =
                                                        initial_date.clone();
                                                    let it =
                                                        initial_time.clone();
                                                    move |_| {
                                                        picked_date.set(
                                                            id.clone(),
                                                        );
                                                        picked_time.set(
                                                            it.clone(),
                                                        );
                                                        set_popover_open
                                                            .update(
                                                                |o| *o = !*o,
                                                            );
                                                    }
                                                }
                                            >
                                                <svg
                                                    xmlns="http://www.w3.org/2000/svg"
                                                    width="12" height="12"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="2"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                >
                                                    <rect
                                                        x="3" y="4"
                                                        width="18"
                                                        height="18"
                                                        rx="2" ry="2"
                                                    />
                                                    <line
                                                        x1="16" y1="2"
                                                        x2="16" y2="6"
                                                    />
                                                    <line
                                                        x1="8" y1="2"
                                                        x2="8" y2="6"
                                                    />
                                                    <line
                                                        x1="3" y1="10"
                                                        x2="21" y2="10"
                                                    />
                                                </svg>
                                                {display}
                                                <span
                                                    class="hover:text-text-primary \
                                                           ml-0.5 \
                                                           cursor-pointer"
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                        if let Some(cb) =
                                                            on_clear_start_at
                                                        {
                                                            cb.run(task_id);
                                                        }
                                                    }
                                                >
                                                    "\u{00d7}"
                                                </span>
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <button
                                                class="inline-flex \
                                                       items-center \
                                                       gap-1 \
                                                       text-text-tertiary \
                                                       hover:text-text-secondary \
                                                       hover:bg-bg-tertiary \
                                                       px-1.5 py-0.5 \
                                                       rounded \
                                                       transition-colors \
                                                       opacity-0 \
                                                       group-hover:opacity-100"
                                                on:click=move |_| {
                                                    picked_date.set(
                                                        String::new(),
                                                    );
                                                    picked_time.set(
                                                        "09:00".to_string(),
                                                    );
                                                    set_popover_open
                                                        .update(
                                                            |o| *o = !*o,
                                                        );
                                                }
                                            >
                                                <svg
                                                    xmlns="http://www.w3.org/2000/svg"
                                                    width="12" height="12"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="2"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                >
                                                    <rect
                                                        x="3" y="4"
                                                        width="18"
                                                        height="18"
                                                        rx="2" ry="2"
                                                    />
                                                    <line
                                                        x1="16" y1="2"
                                                        x2="16" y2="6"
                                                    />
                                                    <line
                                                        x1="8" y1="2"
                                                        x2="8" y2="6"
                                                    />
                                                    <line
                                                        x1="3" y1="10"
                                                        x2="21" y2="10"
                                                    />
                                                </svg>
                                                "Date"
                                            </button>
                                        }.into_any()
                                    }}
                                    // Date/time popover
                                    <Show when=move || popover_open.get()>
                                        <div
                                            class="fixed inset-0 z-40"
                                            on:click=move |_| {
                                                set_popover_open
                                                    .set(false);
                                            }
                                        />
                                        <div class="absolute top-full \
                                                    left-0 mt-1 z-50 \
                                                    bg-bg-secondary \
                                                    border border-border \
                                                    rounded-lg shadow-lg \
                                                    p-3 w-[220px]">
                                            <div class="flex flex-col \
                                                        gap-2">
                                                <label class="text-xs \
                                                    text-text-secondary">
                                                    "Date"
                                                </label>
                                                <input
                                                    type="date"
                                                    class="bg-bg-input \
                                                           border \
                                                           border-border \
                                                           rounded px-2 \
                                                           py-1.5 text-sm \
                                                           text-text-primary \
                                                           w-full \
                                                           focus:outline-none \
                                                           focus:border-accent"
                                                    bind:value=picked_date
                                                    on:change:target=move |ev| {
                                                        picked_date.set(
                                                            ev.target().value(),
                                                        );
                                                    }
                                                />
                                                <label class="text-xs \
                                                    text-text-secondary">
                                                    "Time"
                                                </label>
                                                <input
                                                    type="time"
                                                    class="bg-bg-input \
                                                           border \
                                                           border-border \
                                                           rounded px-2 \
                                                           py-1.5 text-sm \
                                                           text-text-primary \
                                                           w-full \
                                                           focus:outline-none \
                                                           focus:border-accent"
                                                    bind:value=picked_time
                                                    on:change:target=move |ev| {
                                                        picked_time.set(
                                                            ev.target().value(),
                                                        );
                                                    }
                                                />
                                                <div class="flex \
                                                            items-center \
                                                            gap-2 mt-1 \
                                                            pt-2 border-t \
                                                            border-border">
                                                    {has_start_at
                                                        .then(|| {
                                                        view! {
                                                            <button
                                                                class="\
                                                                    text-xs \
                                                                    text-text-tertiary \
                                                                    hover:text-accent \
                                                                    transition-colors"
                                                                on:click=move |_| {
                                                                    set_popover_open
                                                                        .set(
                                                                            false,
                                                                        );
                                                                    if let Some(
                                                                        cb,
                                                                    ) =
                                                                        on_clear_start_at
                                                                    {
                                                                        cb.run(
                                                                            task_id,
                                                                        );
                                                                    }
                                                                }
                                                            >
                                                                "Remove"
                                                            </button>
                                                        }
                                                    })}
                                                    <div class="flex-1" />
                                                    <button
                                                        class="text-xs \
                                                            text-text-secondary \
                                                            hover:text-text-primary \
                                                            px-2 py-1 \
                                                            rounded \
                                                            transition-colors"
                                                        on:click=move |_| {
                                                            set_popover_open
                                                                .set(false);
                                                        }
                                                    >
                                                        "Cancel"
                                                    </button>
                                                    <button
                                                        class="text-xs \
                                                            bg-accent \
                                                            hover:bg-accent-hover \
                                                            text-white \
                                                            px-3 py-1 \
                                                            rounded \
                                                            transition-colors"
                                                        on:click=move |_| {
                                                            let d =
                                                                picked_date
                                                                .get_untracked();
                                                            let t =
                                                                picked_time
                                                                .get_untracked();
                                                            if !d.is_empty() {
                                                                let time =
                                                                    if t
                                                                        .is_empty()
                                                                    {
                                                                        "09:00"
                                                                        .to_string()
                                                                    } else {
                                                                        t
                                                                    };
                                                                let val =
                                                                    format!(
                                                                    "{d}T{time}",
                                                                );
                                                                set_popover_open
                                                                    .set(
                                                                        false,
                                                                    );
                                                                if let Some(
                                                                    cb,
                                                                ) =
                                                                    on_set_start_at
                                                                {
                                                                    cb.run((
                                                                        task_id,
                                                                        val,
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                    >
                                                        "Save"
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    </Show>
                                </div>

                                {project_title.clone().map(|p| {
                                    view! {
                                        <span class="text-text-secondary">
                                            {p}
                                        </span>
                                    }
                                })}
                                {due_date.map(|d| {
                                    view! {
                                        <span>{format!("Due {d}")}</span>
                                    }
                                })}
                                <Show when=move || is_completed.get()>
                                    <span>"Completed"</span>
                                </Show>
                                {tags.clone()
                                    .into_iter()
                                    .map(|tag| {
                                        view! {
                                            <span class="bg-bg-tertiary \
                                                         text-text-secondary \
                                                         text-xs px-2 \
                                                         py-0.5 \
                                                         rounded-full">
                                                {tag}
                                            </span>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
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
                                on_cancel=on_cancel_edit
                            />
                        </div>
                    }
                }
            }
        </Show>
    }
}
