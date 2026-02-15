use leptos::prelude::*;
use north_stores::TaskDetailModalStore;

use crate::components::date_picker::DateTimePicker;
use crate::components::project_picker::ProjectPicker;
use crate::components::subtask_list::SubtaskList;
use crate::components::tag_picker::TagPicker;
use north_ui::{Checkbox, Icon, IconKind, MarkdownView};

#[component]
pub fn TaskDetailModalView(store: TaskDetailModalStore) -> impl IntoView {
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
                    store.close();
                }
            }
        >
            <div
                class="absolute inset-0 bg-black/50"
                on:click=move |_| store.close()
            />
            <div
                class="relative border border-border/60 \
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
                    let is_completed = task.completed_at.is_some();
                    let sequential_limit = task.sequential_limit;

                    let (completed_sig, set_completed_sig) = signal(is_completed);

                    set_title_draft.set(title.clone());
                    set_body_draft.set(body.clone().unwrap_or_default());
                    set_editing_title.set(false);
                    set_editing_body.set(false);

                    Some(view! {
                        // Header
                        <div class="flex items-center justify-between \
                                    px-4 py-3 border-b border-border \
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
                                        <span class="text-text-secondary">
                                            {pt}
                                        </span>
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
                                    <div class="pt-0.5">
                                        <Checkbox
                                            checked=completed_sig
                                            on_toggle=Callback::new(
                                                move |()| {
                                                    let was = completed_sig
                                                        .get_untracked();
                                                    set_completed_sig
                                                        .set(!was);
                                                    store.toggle_complete();
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
                                                        store.update(t, b);
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
                                                    store.update(t, b);
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
                                                store.update(t, b);
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
                                        on_navigate_to=Callback::new(move |id| {
                                            store.navigate_to_subtask(id)
                                        })
                                        on_parent_refetch=Callback::new(move |()| {})
                                    />
                                </div>
                            </div>

                            // Right sidebar
                            <div class="w-52 border-l border-border \
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
            <div class="text-[11px] text-text-tertiary mb-0.5 \
                        uppercase tracking-wide">
                {label}
            </div>
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
