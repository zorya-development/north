use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::date_picker::DateTimePicker;
use crate::components::project_picker::ProjectPicker;
use crate::components::subtask_list::SubtaskList;
use crate::components::tag_picker::TagPicker;
use crate::stores::lookup_store::LookupStore;
use north_ui::{Checkbox, Icon, IconKind, MarkdownView};

#[component]
pub fn TaskDetailModalView(
    task_detail: Resource<Result<Option<TaskWithMeta>, ServerFnError>>,
    ancestors: Resource<Result<Vec<(i64, String, i64)>, ServerFnError>>,
    has_stack: Memo<bool>,
    on_close: Callback<()>,
    on_prev: Callback<()>,
    on_next: Callback<()>,
    on_navigate_to_subtask: Callback<i64>,
    on_navigate_to_ancestor: Callback<i64>,
    on_toggle_complete: Callback<(i64, bool)>,
    on_delete: Callback<i64>,
    on_update: Callback<(i64, String, Option<String>)>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    on_set_tags: Callback<(i64, Vec<String>)>,
    on_set_due_date: Callback<(i64, String)>,
    on_clear_due_date: Callback<i64>,
    on_set_column: Callback<(i64, i64)>,
    on_clear_column: Callback<i64>,
    on_set_seq_limit: Callback<(i64, i16)>,
    on_refetch_detail: Callback<()>,
) -> impl IntoView {
    let (editing_title, set_editing_title) = signal(false);
    let (editing_body, set_editing_body) = signal(false);
    let (title_draft, set_title_draft) = signal(String::new());
    let (body_draft, set_body_draft) = signal(String::new());

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center"
            on:keydown=move |ev| {
                if ev.key() == "Escape"
                    && !editing_title.get_untracked()
                    && !editing_body.get_untracked()
                {
                    on_close.run(());
                }
            }
        >
            <div
                class="absolute inset-0 bg-backdrop"
                on:click=move |_| on_close.run(())
            />
            <div class="relative z-10 bg-bg-secondary border border-border/60 \
                        rounded-2xl shadow-2xl max-w-3xl w-full mx-4 \
                        max-h-[85vh] flex flex-col">
                <Suspense fallback=move || {
                    view! {
                        <div class="p-8 text-center text-text-secondary">
                            "Loading..."
                        </div>
                    }
                }>
                    {move || {
                        let on_close = on_close;
                        let on_prev = on_prev;
                        let on_next = on_next;
                        let on_navigate_to_subtask = on_navigate_to_subtask;
                        let on_navigate_to_ancestor = on_navigate_to_ancestor;
                        let on_toggle_complete = on_toggle_complete;
                        let on_delete = on_delete;
                        let on_update = on_update;
                        let on_set_start_at = on_set_start_at;
                        let on_clear_start_at = on_clear_start_at;
                        let on_set_project = on_set_project;
                        let on_clear_project = on_clear_project;
                        let on_set_tags = on_set_tags;
                        let on_set_due_date = on_set_due_date;
                        let on_clear_due_date = on_clear_due_date;
                        let on_set_column = on_set_column;
                        let on_clear_column = on_clear_column;
                        let on_set_seq_limit = on_set_seq_limit;
                        let on_refetch_detail = on_refetch_detail;

                        Suspend::new(async move {
                            let task_data = task_detail.await;
                            let ancestor_data = ancestors.await;

                            let task = match task_data {
                                Ok(Some(t)) => t,
                                _ => return view! {
                                    <div class="p-8 text-center text-text-secondary">
                                        "Task not found."
                                    </div>
                                }.into_any(),
                            };

                            let ancestor_list = ancestor_data.unwrap_or_default();

                            let task_id = task.task.id;
                            let title = task.task.title.clone();
                            let body = task.task.body.clone();
                            let project_id = task.task.project_id;
                            let project_title = task.project_title.clone();
                            let column_id = task.task.column_id;
                            let column_name = task.column_name.clone();
                            let tags = task.tags.clone();
                            let start_at = task.task.start_at;
                            let due_date = task.task.due_date;
                            let is_completed = task.task.completed_at.is_some();
                            let sequential_limit = task.task.sequential_limit;
                            let _subtask_count = task.subtask_count;
                            let _has_parent = task.task.parent_id.is_some();

                            let (completed_sig, set_completed_sig) = signal(is_completed);

                            // Init drafts
                            set_title_draft.set(title.clone());
                            set_body_draft.set(body.clone().unwrap_or_default());
                            set_editing_title.set(false);
                            set_editing_body.set(false);

                            let has_stack_val = has_stack.get();

                            view! {
                                // Header
                                <div class="flex items-center justify-between \
                                            px-4 py-3 border-b border-border \
                                            flex-shrink-0">
                                    <div class="flex items-center gap-1 \
                                                text-xs text-text-tertiary \
                                                min-w-0 truncate">
                                        {project_title.clone().map(|pt| {
                                            view! {
                                                <span class="text-text-secondary">
                                                    {pt}
                                                </span>
                                            }
                                        })}
                                        {column_name.clone().map(|cn| {
                                            view! {
                                                <>
                                                    {project_title.as_ref().map(|_| {
                                                        view! {
                                                            <span class="mx-1">"/"</span>
                                                        }
                                                    })}
                                                    <span>{cn}</span>
                                                </>
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
                                            on:click=move |_| on_prev.run(())
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
                                            on:click=move |_| on_next.run(())
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
                                            on:click=move |_| on_delete.run(task_id)
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
                                            on:click=move |_| on_close.run(())
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
                                                    border-b border-border \
                                                    flex-shrink-0 overflow-x-auto">
                                            {ancestor_list.into_iter().map(
                                                |(aid, atitle, acount)| {
                                                    view! {
                                                        <button
                                                            class="hover:text-text-primary \
                                                                   transition-colors \
                                                                   whitespace-nowrap"
                                                            on:click=move |_| {
                                                                on_navigate_to_ancestor.run(aid)
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
                                            <div class="pt-0.5">
                                                <Checkbox
                                                    checked=completed_sig
                                                    on_toggle=Callback::new(
                                                        move |()| {
                                                            let was = completed_sig
                                                                .get_untracked();
                                                            set_completed_sig
                                                                .set(!was);
                                                            on_toggle_complete
                                                                .run((
                                                                    task_id, was,
                                                                ));
                                                        },
                                                    )
                                                    checked_label="Mark incomplete"
                                                    unchecked_label="Complete task"
                                                />
                                            </div>
                                            <Show
                                                when=move || editing_title.get()
                                                fallback=move || {
                                                    view! {
                                                        <h2
                                                            class="text-lg font-semibold \
                                                                   text-text-primary \
                                                                   cursor-pointer \
                                                                   hover:bg-bg-tertiary \
                                                                   rounded px-1 -mx-1 \
                                                                   flex-1"
                                                            on:click=move |_| {
                                                                set_editing_title
                                                                    .set(true);
                                                            }
                                                        >
                                                            {move || title_draft.get()}
                                                        </h2>
                                                    }
                                                }
                                            >
                                                <input
                                                    type="text"
                                                    class="text-lg font-semibold \
                                                           text-text-primary \
                                                           bg-bg-input border \
                                                           border-border rounded \
                                                           px-1 -mx-1 flex-1 \
                                                           w-full \
                                                           focus:outline-none \
                                                           focus:border-accent"
                                                    prop:value=move || {
                                                        title_draft.get()
                                                    }
                                                    on:input=move |ev| {
                                                        set_title_draft
                                                            .set(
                                                                event_target_value(
                                                                    &ev,
                                                                ),
                                                            );
                                                    }
                                                    on:keydown=move |ev| {
                                                        match ev.key().as_str() {
                                                            "Enter" => {
                                                                ev.prevent_default();
                                                                set_editing_title
                                                                    .set(false);
                                                                let t =
                                                                    title_draft
                                                                        .get_untracked();
                                                                let b =
                                                                    body_draft
                                                                        .get_untracked();
                                                                let b = if b
                                                                    .trim()
                                                                    .is_empty()
                                                                {
                                                                    None
                                                                } else {
                                                                    Some(b)
                                                                };
                                                                on_update.run((
                                                                    task_id, t, b,
                                                                ));
                                                            }
                                                            "Escape" => {
                                                                set_editing_title
                                                                    .set(false);
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                    on:blur=move |_| {
                                                        if editing_title
                                                            .get_untracked()
                                                        {
                                                            set_editing_title
                                                                .set(false);
                                                            let t =
                                                                title_draft
                                                                    .get_untracked();
                                                            let b =
                                                                body_draft
                                                                    .get_untracked();
                                                            let b = if b
                                                                .trim()
                                                                .is_empty()
                                                            {
                                                                None
                                                            } else {
                                                                Some(b)
                                                            };
                                                            on_update.run((
                                                                task_id, t, b,
                                                            ));
                                                        }
                                                    }
                                                    node_ref={
                                                        let r = NodeRef::<
                                                            leptos::html::Input,
                                                        >::new();
                                                        Effect::new(move || {
                                                            if editing_title.get()
                                                            {
                                                                if let Some(el) =
                                                                    r.get()
                                                                {
                                                                    let _ = el
                                                                        .focus();
                                                                }
                                                            }
                                                        });
                                                        r
                                                    }
                                                />
                                            </Show>
                                        </div>

                                        // Body
                                        <div class="ml-6">
                                            <Show
                                                when=move || editing_body.get()
                                                fallback=move || {
                                                    view! {
                                                        <div
                                                            class="cursor-pointer \
                                                                   hover:bg-bg-tertiary \
                                                                   rounded p-1 -m-1 \
                                                                   min-h-[2rem]"
                                                            on:click=move |_| {
                                                                set_editing_body
                                                                    .set(true);
                                                            }
                                                        >
                                                            {move || {
                                                                let bd = body_draft.get();
                                                                if bd.trim().is_empty() {
                                                                    view! {
                                                                        <span class="text-sm \
                                                                                    text-text-tertiary \
                                                                                    italic">
                                                                            "Add description..."
                                                                        </span>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <MarkdownView content=bd/>
                                                                    }.into_any()
                                                                }
                                                            }}
                                                        </div>
                                                    }
                                                }
                                            >
                                                <textarea
                                                    class="w-full text-sm \
                                                           text-text-primary \
                                                           bg-bg-input border \
                                                           border-border \
                                                           rounded p-2 \
                                                           focus:outline-none \
                                                           focus:border-accent \
                                                           resize-y \
                                                           min-h-[4rem]"
                                                    prop:value=move || {
                                                        body_draft.get()
                                                    }
                                                    on:input=move |ev| {
                                                        set_body_draft.set(
                                                            event_target_value(&ev),
                                                        );
                                                    }
                                                    on:blur=move |_| {
                                                        set_editing_body.set(false);
                                                        let t = title_draft.get_untracked();
                                                        let b = body_draft.get_untracked();
                                                        let b = if b.trim().is_empty() {
                                                            None
                                                        } else {
                                                            Some(b)
                                                        };
                                                        on_update.run((task_id, t, b));
                                                    }
                                                    on:keydown=move |ev| {
                                                        if ev.key() == "Escape" {
                                                            set_editing_body.set(false);
                                                        }
                                                    }
                                                    node_ref={
                                                        let r = NodeRef::<
                                                            leptos::html::Textarea,
                                                        >::new();
                                                        Effect::new(move || {
                                                            if editing_body.get() {
                                                                if let Some(el) = r.get() {
                                                                    let _ = el.focus();
                                                                }
                                                            }
                                                        });
                                                        r
                                                    }
                                                />
                                            </Show>
                                        </div>

                                        // Subtask area
                                        <div class="ml-6">
                                            <SubtaskList
                                                parent_id=task_id
                                                parent_sequential_limit=sequential_limit
                                                depth=0
                                                project_id=project_id
                                                on_navigate_to=on_navigate_to_subtask
                                                on_parent_refetch=on_refetch_detail
                                            />
                                        </div>
                                    </div>

                                    // Right sidebar
                                    <div class="w-56 border-l border-border p-4 \
                                                space-y-3 overflow-y-auto \
                                                flex-shrink-0">
                                        // Project
                                        <SidebarRow label="Project">
                                            <ProjectPicker
                                                task_id=task_id
                                                project_id=project_id
                                                project_title=project_title
                                                on_set_project=on_set_project
                                                on_clear_project=on_clear_project
                                            />
                                        </SidebarRow>

                                        // Column
                                        <SidebarRow label="Column">
                                            <ColumnPicker
                                                task_id=task_id
                                                project_id=project_id
                                                column_id=column_id
                                                column_name=column_name
                                                on_set_column=on_set_column
                                                on_clear_column=on_clear_column
                                            />
                                        </SidebarRow>

                                        // Tags
                                        <SidebarRow label="Tags">
                                            <TagPicker
                                                task_id=task_id
                                                tags=tags
                                                on_set_tags=on_set_tags
                                            />
                                        </SidebarRow>

                                        // Start date
                                        <SidebarRow label="Start date">
                                            <DateTimePicker
                                                task_id=task_id
                                                start_at=start_at
                                                on_set_start_at=on_set_start_at
                                                on_clear_start_at=on_clear_start_at
                                            />
                                        </SidebarRow>

                                        // Due date
                                        <SidebarRow label="Due date">
                                            <DueDatePicker
                                                task_id=task_id
                                                due_date=due_date
                                                on_set_due_date=on_set_due_date
                                                on_clear_due_date=on_clear_due_date
                                            />
                                        </SidebarRow>

                                        // Sequential limit
                                        <SidebarRow label="Seq. limit">
                                            <SequentialLimitInput
                                                task_id=task_id
                                                sequential_limit=sequential_limit
                                                on_set_seq_limit=on_set_seq_limit
                                            />
                                        </SidebarRow>
                                    </div>
                                </div>
                            }.into_any()
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn SidebarRow(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div>
            <div class="text-xs text-text-tertiary mb-1">{label}</div>
            {children()}
        </div>
    }
}

#[component]
fn ColumnPicker(
    task_id: i64,
    project_id: Option<i64>,
    column_id: Option<i64>,
    column_name: Option<String>,
    on_set_column: Callback<(i64, i64)>,
    on_clear_column: Callback<i64>,
) -> impl IntoView {
    let lookup = use_context::<LookupStore>();
    let (open, set_open) = signal(false);

    // Pre-clone the store so we can move it into closures
    let columns_resource = lookup.map(|s| s.columns);

    view! {
        <div class="relative">
            <button
                class="text-sm text-text-secondary hover:text-text-primary \
                       transition-colors"
                on:click=move |_| set_open.update(|o| *o = !*o)
            >
                {column_name.unwrap_or_else(|| "None".to_string())}
            </button>
            <Show when=move || open.get()>
                {move || {
                    view! {
                        <div class="absolute top-full left-0 mt-1 z-50 \
                                    bg-bg-secondary border border-border \
                                    rounded shadow-lg min-w-[10rem] py-1">
                            <button
                                class="w-full text-left px-3 py-1 text-sm \
                                       text-text-tertiary hover:bg-bg-tertiary \
                                       transition-colors"
                                on:click=move |_| {
                                    on_clear_column.run(task_id);
                                    set_open.set(false);
                                }
                            >
                                "None"
                            </button>
                            {columns_resource.map(|res| {
                                view! {
                                    <Suspense fallback=|| ()>
                                        {move || {
                                            Suspend::new(async move {
                                                let cols = res.await;
                                                match cols {
                                                    Ok(cols) => {
                                                        let filtered: Vec<_> = cols
                                                            .into_iter()
                                                            .filter(|c| {
                                                                project_id
                                                                    .map(|pid| {
                                                                        c.project_id == pid
                                                                    })
                                                                    .unwrap_or(false)
                                                            })
                                                            .collect();
                                                        view! {
                                                            <div>
                                                                {filtered
                                                                    .into_iter()
                                                                    .map(|col| {
                                                                        let cid = col.id;
                                                                        let is_current = column_id
                                                                            == Some(cid);
                                                                        view! {
                                                                            <button
                                                                                class=if is_current {
                                                                                    "w-full text-left px-3 \
                                                                                     py-1 text-sm \
                                                                                     text-accent \
                                                                                     bg-bg-tertiary"
                                                                                } else {
                                                                                    "w-full text-left px-3 \
                                                                                     py-1 text-sm \
                                                                                     text-text-primary \
                                                                                     hover:bg-bg-tertiary \
                                                                                     transition-colors"
                                                                                }
                                                                                on:click=move |_| {
                                                                                    on_set_column.run((
                                                                                        task_id, cid,
                                                                                    ));
                                                                                    set_open.set(false);
                                                                                }
                                                                            >
                                                                                {col.name}
                                                                            </button>
                                                                        }
                                                                    })
                                                                    .collect::<Vec<_>>()}
                                                            </div>
                                                        }.into_any()
                                                    }
                                                    Err(_) => view! {
                                                        <div class="px-3 py-1 text-xs \
                                                                    text-text-tertiary">
                                                            "Error loading columns"
                                                        </div>
                                                    }.into_any(),
                                                }
                                            })
                                        }}
                                    </Suspense>
                                }
                            })}
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}

#[component]
fn DueDatePicker(
    task_id: i64,
    due_date: Option<chrono::NaiveDate>,
    on_set_due_date: Callback<(i64, String)>,
    on_clear_due_date: Callback<i64>,
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
                        on_set_due_date.run((task_id, val));
                    }
                }
            />
            {due_date.map(|_| {
                view! {
                    <button
                        class="p-0.5 text-text-tertiary hover:text-text-primary \
                               transition-colors flex-shrink-0"
                        on:click=move |_| on_clear_due_date.run(task_id)
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
fn SequentialLimitInput(
    task_id: i64,
    sequential_limit: i16,
    on_set_seq_limit: Callback<(i64, i16)>,
) -> impl IntoView {
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
                        on_set_seq_limit.run((task_id, n));
                    }
                }
            }
        />
    }
}
