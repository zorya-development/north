use leptos::prelude::*;
use north_stores::{use_app_store, IdFilter, TaskDetailModalStore, TaskStoreFilter};

use crate::atoms::{Text, TextColor, TextVariant};
use crate::components::date_picker::DateTimePicker;
use crate::components::recurrence_modal::RecurrenceModal;
use crate::containers::inline_task_input::InlineTaskInput;
use crate::containers::project_picker::ProjectPicker;
use crate::containers::tag_picker::TagPicker;
use crate::containers::task_checkbox::TaskCheckbox;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::{ExtraVisibleIds, TraversableTaskList};
use north_ui::{Icon, IconKind};

#[component]
pub fn TaskDetailModalView(
    store: TaskDetailModalStore,
    on_recurrence_open: Callback<()>,
    on_recurrence_close: Callback<()>,
    show_recurrence_modal: Signal<bool>,
) -> impl IntoView {
    let app_store = use_app_store();
    let (title_draft, set_title_draft) = signal(String::new());
    let (body_draft, set_body_draft) = signal(String::new());
    let subtask_show_completed = RwSignal::new(false);
    let (show_inline_input, set_show_inline_input) = signal(false);
    let input_value = RwSignal::new(String::new());
    let extra_visible_ids = expect_context::<ExtraVisibleIds>().0;
    let title_input_ref = NodeRef::<leptos::html::Input>::new();
    let subtask_cursor = RwSignal::new(None::<i64>);
    let focused_task_id = RwSignal::new(None::<i64>);
    let subtask_item_config = ItemConfig {
        show_project: false,
        ..Default::default()
    };

    let save = move || {
        let t = title_draft.get_untracked();
        let b = body_draft.get_untracked();
        let b = if b.trim().is_empty() { None } else { Some(b) };
        store.update(t, b);
    };

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center">
            <div
                class="absolute inset-0 bg-black/50"
                on:click=move |_| store.close()
            />
            <div
                role="dialog"
                class="relative border border-(--border-muted) \
                       rounded-2xl shadow-2xl max-w-3xl w-full mx-4 \
                       max-h-[85vh] flex flex-col"
                style="background-color: var(--bg-secondary)"
            >
                {move || {
                    let task = store.task()?;
                    let ancestor_list = store.ancestors();
                    let has_stack_val = store.has_stack();

                    let task_id = task.id;
                    let title = task.title.clone();
                    let body = task.body.clone();
                    let project_id = task.project_id;
                    let project_title = task.project_title.clone();
                    let tags = task.tags.clone();
                    let start_at = task.start_at;
                    let due_date = task.due_date;
                    let sequential_limit = task.sequential_limit;
                    let recurrence_type = task.recurrence_type;
                    let recurrence_rule = task.recurrence_rule.clone();

                    set_title_draft.set(title.clone());
                    set_body_draft.set(body.clone().unwrap_or_default());

                    if focused_task_id.get_untracked() != Some(task_id) {
                        focused_task_id.set(Some(task_id));
                        request_animation_frame(move || {
                            if let Some(el) = title_input_ref.get() {
                                let _ = el.focus();
                            }
                        });
                    }

                    Some(view! {
                        // Header
                        <div class="flex items-center justify-between \
                                    px-4 py-3 border-b border-(--border-muted) \
                                    flex-shrink-0">
                            <div class="flex items-center gap-1 \
                                        text-xs text-text-tertiary \
                                        min-w-0 truncate">
                                {project_title.clone().map(|pt| {
                                    view! {
                                        <Icon
                                            kind=IconKind::Folder
                                            class="w-3.5 h-3.5 text-text-tertiary \
                                                   flex-shrink-0"
                                        />
                                        <Text variant=TextVariant::BodySm color=TextColor::Secondary>
                                            {pt}
                                        </Text>
                                    }
                                })}
                            </div>
                            <div class="flex items-center gap-1">
                                <button
                                    class="p-1 rounded text-text-tertiary \
                                           hover:text-text-primary \
                                           hover:bg-bg-tertiary \
                                           transition-colors \
                                           disabled:opacity-30"
                                    on:click=move |_| store.prev()
                                    disabled=has_stack_val
                                    title="Previous task"
                                >
                                    <Icon
                                        kind=IconKind::ChevronLeft
                                        class="w-4 h-4"
                                    />
                                </button>
                                <button
                                    class="p-1 rounded text-text-tertiary \
                                           hover:text-text-primary \
                                           hover:bg-bg-tertiary \
                                           transition-colors \
                                           disabled:opacity-30"
                                    on:click=move |_| store.next()
                                    disabled=has_stack_val
                                    title="Next task"
                                >
                                    <Icon
                                        kind=IconKind::ChevronRight
                                        class="w-4 h-4"
                                    />
                                </button>
                                <button
                                    class="p-1 rounded text-danger \
                                           hover:text-danger-hover \
                                           hover:bg-bg-tertiary \
                                           transition-colors"
                                    on:click=move |_| store.delete()
                                    title="Delete task"
                                >
                                    <Icon
                                        kind=IconKind::Trash
                                        class="w-4 h-4"
                                    />
                                </button>
                                <button
                                    class="p-1 rounded text-text-tertiary \
                                           hover:text-text-primary \
                                           hover:bg-bg-tertiary \
                                           transition-colors"
                                    on:click=move |_| store.close()
                                    title="Close"
                                >
                                    <Icon
                                        kind=IconKind::Close
                                        class="w-4 h-4"
                                    />
                                </button>
                            </div>
                        </div>

                        // Breadcrumb bar (when viewing subtask)
                        {(!ancestor_list.is_empty()).then(|| {
                            let ancestor_list = ancestor_list.clone();
                            view! {
                                <div class="flex items-center gap-1 \
                                            px-4 py-2 text-xs \
                                            text-text-tertiary \
                                            border-b border-(--border-muted) \
                                            flex-shrink-0 overflow-x-auto">
                                    {ancestor_list.into_iter().map(
                                        |(aid, atitle, acount)| {
                                            view! {
                                                <button
                                                    class="hover:text-text-primary \
                                                           transition-colors \
                                                           whitespace-nowrap"
                                                    on:click=move |_| {
                                                        store.navigate_to_ancestor(aid)
                                                    }
                                                >
                                                    {atitle}
                                                </button>
                                                <span class="text-text-tertiary">
                                                    {format!(" | {acount}")}
                                                </span>
                                                <Icon
                                                    kind=IconKind::ChevronRight
                                                    class="w-3 h-3"
                                                />
                                            }
                                        },
                                    ).collect::<Vec<_>>()}
                                </div>
                            }
                        })}

                        // Body
                        <div class="flex flex-1 min-h-0 overflow-hidden">
                            // Left column
                            <div class="flex-1 overflow-y-auto p-4 \
                                        space-y-4">
                                // Title
                                <div class="flex items-start gap-2">
                                    <div class="pt-1">
                                        <TaskCheckbox task_id=task_id/>
                                    </div>
                                    <input
                                        type="text"
                                        node_ref=title_input_ref
                                        class="text-lg font-semibold \
                                               text-text-primary \
                                               bg-transparent \
                                               border-none \
                                               px-1 -mx-1 flex-1 \
                                               w-full \
                                               focus:outline-none \
                                               no-focus-ring"
                                        prop:value=move || {
                                            title_draft.get()
                                        }
                                        on:input=move |ev| {
                                            set_title_draft
                                                .set(event_target_value(&ev));
                                        }
                                        on:keydown=move |ev| {
                                            if ev.key() == "Enter" {
                                                ev.prevent_default();
                                                save();
                                            }
                                        }
                                        on:blur=move |_| {
                                            save();
                                        }
                                    />
                                </div>

                                // Body
                                <div class="ml-6">
                                    <textarea
                                        class="w-full text-sm \
                                               text-text-primary \
                                               bg-transparent \
                                               border-none \
                                               p-1 -m-1 \
                                               focus:outline-none \
                                               no-focus-ring \
                                               resize-none \
                                               min-h-[2rem] \
                                               placeholder:text-text-tertiary \
                                               placeholder:italic"
                                        placeholder="Add description..."
                                        prop:value=move || {
                                            body_draft.get()
                                        }
                                        on:input=move |ev| {
                                            set_body_draft.set(
                                                event_target_value(&ev),
                                            );
                                        }
                                        on:blur=move |_| {
                                            save();
                                        }
                                    />
                                </div>

                                // Subtask area
                                {
                                    let all_subtasks = app_store
                                        .tasks
                                        .filtered(TaskStoreFilter {
                                            parent_id: IdFilter::Is(task_id),
                                            ..Default::default()
                                        });
                                    let subtask_ids = Memo::new(move |_| {
                                        all_subtasks
                                            .get()
                                            .iter()
                                            .map(|t| t.id)
                                            .collect::<Vec<_>>()
                                    });
                                    let completed_count = Memo::new(move |_| {
                                        all_subtasks
                                            .get()
                                            .iter()
                                            .filter(|t| t.completed_at.is_some())
                                            .count()
                                    });
                                    let total_count = Memo::new(move |_| {
                                        all_subtasks.get().len()
                                    });
                                    let default_project_signal =
                                        Signal::derive(move || project_id);

                                    view! {
                                        <div class="ml-6">
                                            <TraversableTaskList
                                                root_task_ids=subtask_ids
                                                show_completed=subtask_show_completed
                                                scoped=true
                                                item_config=subtask_item_config
                                                is_loaded=Signal::derive(|| true)
                                                on_task_click=Callback::new(
                                                    move |id| {
                                                        store.navigate_to_subtask(id)
                                                    },
                                                )
                                                on_reorder=Callback::new(
                                                    move |(id, key, parent)| {
                                                        app_store
                                                            .tasks
                                                            .reorder_task(id, key, parent)
                                                    },
                                                )
                                                default_project_id=default_project_signal
                                                empty_message="No subtasks."
                                                allow_reorder=true
                                                cursor_task_id=subtask_cursor
                                            />
                                            // Inline task input for mouse-friendly subtask creation
                                            <Show when=move || show_inline_input.get()>
                                                <InlineTaskInput
                                                    parent_id=task_id
                                                    value=input_value
                                                    on_created=Callback::new(
                                                        move |id| {
                                                            extra_visible_ids.update(|ids| {
                                                                if !ids.contains(&id) {
                                                                    ids.push(id);
                                                                }
                                                            });
                                                        },
                                                    )
                                                    on_close=Callback::new(
                                                        move |()| {
                                                            set_show_inline_input.set(false);
                                                        },
                                                    )
                                                />
                                            </Show>
                                            <Show when=move || !show_inline_input.get()>
                                                <button
                                                    class="my-3 text-xs text-accent \
                                                           hover:text-accent-hover \
                                                           hover:underline cursor-pointer \
                                                           transition-colors"
                                                    on:click=move |_| {
                                                        set_show_inline_input.set(true);
                                                    }
                                                >
                                                    "+ Add subtask"
                                                </button>
                                            </Show>
                                            // Toggle bar
                                            <Show when=move || {
                                                completed_count.get() > 0usize
                                            }>
                                                <div class="py-1 flex items-center \
                                                            gap-2 text-xs">
                                                    <button
                                                        class="text-accent \
                                                               hover:text-accent-hover \
                                                               hover:underline \
                                                               cursor-pointer \
                                                               transition-colors"
                                                        on:click=move |_| {
                                                            subtask_show_completed
                                                                .update(|v| *v = !*v);
                                                        }
                                                    >
                                                        {move || {
                                                            if subtask_show_completed.get() {
                                                                "Hide Completed".to_string()
                                                            } else {
                                                                format!(
                                                                    "Show Completed ({})",
                                                                    completed_count.get(),
                                                                )
                                                            }
                                                        }}
                                                    </button>
                                                    <span class="text-text-tertiary">
                                                        {move || format!(
                                                            "Total: {}",
                                                            total_count.get(),
                                                        )}
                                                    </span>
                                                </div>
                                            </Show>
                                        </div>
                                    }
                                }
                            </div>

                            // Right sidebar
                            <div class="w-52 border-l border-(--border-muted) \
                                        px-3 py-3 space-y-2 \
                                        overflow-y-auto flex-shrink-0">
                                // Project
                                <SidebarRow label="Project">
                                    <ProjectPicker
                                        task_id=task_id
                                        project_id=project_id
                                        project_title=project_title.clone()
                                        on_set_project=Callback::new(
                                            move |(_task_id, project_id): (i64, i64)| {
                                                store.set_project(project_id)
                                            },
                                        )
                                        on_clear_project=Callback::new(
                                            move |_id: i64| store.clear_project(),
                                        )
                                        always_visible=true
                                    />
                                </SidebarRow>

                                // Tags
                                <SidebarRow label="Tags">
                                    <TagPicker
                                        task_id=task_id
                                        tags=tags.clone()
                                        on_set_tags=Callback::new(
                                            move |(_task_id, tags): (i64, Vec<String>)| {
                                                store.set_tags(tags)
                                            },
                                        )
                                        always_visible=true
                                    />
                                </SidebarRow>

                                // Start date
                                <SidebarRow label="Start date">
                                    <DateTimePicker
                                        task_id=task_id
                                        start_at=start_at
                                        on_set_start_at=Callback::new(
                                            move |(_id, start_at): (i64, String)| {
                                                store.set_start_at(start_at)
                                            },
                                        )
                                        on_clear_start_at=Callback::new(
                                            move |_id: i64| store.clear_start_at(),
                                        )
                                        always_visible=true
                                    />
                                </SidebarRow>

                                // Due date
                                <SidebarRow label="Due date">
                                    <DueDatePicker
                                        due_date=due_date
                                        store=store
                                    />
                                </SidebarRow>

                                // Recurrence
                                <SidebarRow label="Recurrence">
                                    <RecurrenceSidebarButton
                                        recurrence_type=recurrence_type
                                        recurrence_rule=recurrence_rule.clone()
                                        on_click=Callback::new(move |()| {
                                            on_recurrence_open.run(());
                                        })
                                    />
                                    <Show when=move || show_recurrence_modal.get()>
                                        <RecurrenceModal
                                            recurrence_type=recurrence_type
                                            recurrence_rule=recurrence_rule.clone()
                                            on_save=Callback::new(move |(rt, rr)| {
                                                store.set_recurrence(rt, rr);
                                                on_recurrence_close.run(());
                                            })
                                            on_close=Callback::new(move |()| {
                                                on_recurrence_close.run(());
                                            })
                                        />
                                    </Show>
                                </SidebarRow>

                                // Sequential limit
                                <SidebarRow label="Seq. limit">
                                    <SequentialLimitInput
                                        sequential_limit=sequential_limit
                                        store=store
                                    />
                                </SidebarRow>
                            </div>
                        </div>
                    })
                }}
            </div>
        </div>
    }
}

#[component]
fn SidebarRow(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div>
            <Text variant=TextVariant::LabelSm color=TextColor::Tertiary class="block mb-0.5">
                {label}
            </Text>
            {children()}
        </div>
    }
}

#[component]
fn DueDatePicker(
    due_date: Option<chrono::NaiveDate>,
    store: TaskDetailModalStore,
) -> impl IntoView {
    let display = due_date.map(|d| d.format("%Y-%m-%d").to_string());

    view! {
        <div class="flex items-center gap-1">
            <input
                type="date"
                class="text-sm bg-transparent text-text-secondary \
                       border-none focus:outline-none cursor-pointer \
                       w-full"
                prop:value=move || display.clone().unwrap_or_default()
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    if !val.is_empty() {
                        store.set_due_date(val);
                    }
                }
            />
            {due_date.map(|_| {
                view! {
                    <button
                        class="p-0.5 text-text-tertiary hover:text-text-primary \
                               transition-colors flex-shrink-0"
                        on:click=move |_| store.clear_due_date()
                        title="Clear due date"
                    >
                        <Icon kind=IconKind::Close class="w-3 h-3"/>
                    </button>
                }
            })}
        </div>
    }
}

#[component]
fn RecurrenceSidebarButton(
    recurrence_type: Option<north_dto::RecurrenceType>,
    recurrence_rule: Option<String>,
    on_click: Callback<()>,
) -> impl IntoView {
    let label = match recurrence_type {
        Some(_) => recurrence_rule
            .as_deref()
            .and_then(north_dto::RecurrenceRule::parse)
            .map(|r| r.summarize())
            .unwrap_or_else(|| "None".to_string()),
        None => "None".to_string(),
    };

    view! {
        <button
            class="text-sm text-text-secondary hover:text-text-primary \
                   transition-colors cursor-pointer flex items-center gap-1"
            on:click=move |_| on_click.run(())
        >
            <Icon kind=IconKind::Recurrence class="w-3.5 h-3.5"/>
            {label}
        </button>
    }
}

#[component]
fn SequentialLimitInput(sequential_limit: i16, store: TaskDetailModalStore) -> impl IntoView {
    let (value, set_value) = signal(sequential_limit.to_string());

    view! {
        <input
            type="number"
            min="1"
            max="999"
            class="w-16 text-sm bg-bg-input border border-border \
                   rounded px-2 py-0.5 text-text-primary \
                   focus:outline-none focus:border-accent"
            prop:value=move || value.get()
            on:input=move |ev| {
                set_value.set(event_target_value(&ev));
            }
            on:change=move |ev| {
                let val = event_target_value(&ev);
                if let Ok(n) = val.parse::<i16>() {
                    if n >= 1 {
                        store.set_sequential_limit(n);
                    }
                }
            }
        />
    }
}
