use std::collections::HashSet;

use leptos::prelude::*;
use north_domain::TaskWithMeta;
use north_ui::{Checkbox, Icon, IconKind};

use crate::components::subtask_form::SubtaskForm;

#[component]
pub fn SubtaskListView(
    subtasks: Resource<Result<Vec<TaskWithMeta>, ServerFnError>>,
    parent_id: i64,
    depth: usize,
    project_id: Option<i64>,
    hide_completed: ReadSignal<bool>,
    set_hide_completed: WriteSignal<bool>,
    on_toggle_complete: Callback<(i64, bool)>,
    on_delete: Callback<i64>,
    on_navigate_to: Callback<i64>,
    on_parent_refetch: Callback<()>,
    on_created: Callback<()>,
    parent_sequential_limit: i16,
) -> impl IntoView {
    let _ = parent_sequential_limit;
    let collapsed: RwSignal<HashSet<i64>> = RwSignal::new(HashSet::new());

    view! {
        <Suspense fallback=|| ()>
            {move || {
                let on_toggle_complete = on_toggle_complete;
                let on_delete = on_delete;
                let on_navigate_to = on_navigate_to;
                let on_parent_refetch = on_parent_refetch;
                let on_created = on_created;

                Suspend::new(async move {
                    let tasks = match subtasks.await {
                        Ok(t) => t,
                        Err(_) => return view! {
                            <div class="text-xs text-danger">
                                "Error loading subtasks"
                            </div>
                        }.into_any(),
                    };

                    if tasks.is_empty() && depth > 0 {
                        return view! {
                            <Show when=move || depth < 5>
                                <SubtaskForm
                                    parent_id=parent_id
                                    project_id=project_id
                                    depth=depth
                                    on_created=on_created
                                />
                            </Show>
                        }.into_any();
                    }

                    let total = tasks.len();
                    let completed_count = tasks
                        .iter()
                        .filter(|t| t.task.completed_at.is_some())
                        .count();

                    let _sibling_ids: Vec<i64> =
                        tasks.iter().map(|t| t.task.id).collect();

                    view! {
                        <div class="space-y-0.5">
                            // Header
                            <div class="flex items-center justify-between \
                                        text-xs text-text-tertiary mb-1">
                                <div class="flex items-center gap-1">
                                    <Icon
                                        kind=IconKind::Subtask
                                        class="w-3.5 h-3.5"
                                    />
                                    <span>
                                        {format!(
                                            "Sub-tasks {completed_count}/{total}",
                                        )}
                                    </span>
                                </div>
                                <button
                                    class="hover:text-text-secondary \
                                           transition-colors"
                                    on:click=move |_| {
                                        set_hide_completed.update(|v| *v = !*v);
                                    }
                                >
                                    {move || {
                                        if hide_completed.get() {
                                            "Show completed"
                                        } else {
                                            "Hide completed"
                                        }
                                    }}
                                </button>
                            </div>

                            // Task rows
                            {tasks
                                .into_iter()
                                .map(|task| {
                                    let task_id = task.task.id;
                                    let title = task.task.title.clone();
                                    let is_completed =
                                        task.task.completed_at.is_some();
                                    let actionable = task.actionable;
                                    let sub_count = task.subtask_count;
                                    let seq_limit = task.task.sequential_limit;
                                    let (completed_sig, set_completed_sig) =
                                        signal(is_completed);

                                    let is_collapsed = Memo::new(move |_| {
                                        collapsed.get().contains(&task_id)
                                    });

                                    let row_class = if is_completed {
                                        "line-through text-text-tertiary opacity-60"
                                    } else if !actionable {
                                        "text-text-primary opacity-40"
                                    } else {
                                        "text-text-primary"
                                    };

                                    let pad = format!("pl-{}", depth * 4);

                                    let div_class = format!(
                                        "flex items-center gap-1.5 py-0.5 \
                                         group/subtask {pad}",
                                    );
                                    let title_class = format!(
                                        "text-sm flex-1 cursor-pointer \
                                         hover:underline {row_class}",
                                    );

                                    view! {
                                        <Show when=move || {
                                            !(hide_completed.get() && is_completed)
                                        }>
                                            {
                                                let title = title.clone();
                                                let div_class = div_class.clone();
                                                let title_class = title_class.clone();
                                                view! {
                                                    <div class=div_class>
                                                        // Collapse toggle
                                                        {(sub_count > 0).then(|| {
                                                            view! {
                                                                <button
                                                                    class="p-0.5 \
                                                                           text-text-tertiary \
                                                                           hover:text-text-secondary \
                                                                           transition-colors"
                                                                    on:click=move |_| {
                                                                        collapsed.update(|s| {
                                                                            if s.contains(&task_id) {
                                                                                s.remove(&task_id);
                                                                            } else {
                                                                                s.insert(task_id);
                                                                            }
                                                                        });
                                                                    }
                                                                >
                                                                    <Icon
                                                                        kind=if is_collapsed.get() {
                                                                            IconKind::ChevronRight
                                                                        } else {
                                                                            IconKind::ChevronDown
                                                                        }
                                                                        class="w-3 h-3"
                                                                    />
                                                                </button>
                                                            }
                                                        })}
                                                        {(sub_count == 0).then(|| {
                                                            view! {
                                                                <div class="w-4"/>
                                                            }
                                                        })}

                                                        // Checkbox
                                                        <Checkbox
                                                            checked=completed_sig
                                                            on_toggle=Callback::new(
                                                                move |()| {
                                                                    let was =
                                                                        completed_sig
                                                                            .get_untracked();
                                                                    set_completed_sig
                                                                        .set(!was);
                                                                    on_toggle_complete.run((
                                                                        task_id, was,
                                                                    ));
                                                                },
                                                            )
                                                            checked_label="Mark incomplete"
                                                            unchecked_label="Complete"
                                                        />

                                                        // Title
                                                        <span
                                                            class=title_class
                                                            on:click=move |_| {
                                                                on_navigate_to.run(task_id)
                                                            }
                                                        >
                                                            {title}
                                                        </span>

                                                        // Child count badge
                                                        {(sub_count > 0).then(|| {
                                                            view! {
                                                                <span class="text-xs \
                                                                             text-text-tertiary">
                                                                    {sub_count}
                                                                </span>
                                                            }
                                                        })}

                                                        // Delete
                                                        <button
                                                            class="p-0.5 text-text-tertiary \
                                                                   hover:text-danger \
                                                                   transition-colors \
                                                                   opacity-0 \
                                                                   group-hover/subtask:opacity-100"
                                                            on:click=move |_| {
                                                                on_delete.run(task_id)
                                                            }
                                                            title="Delete"
                                                        >
                                                            <Icon
                                                                kind=IconKind::Trash
                                                                class="w-3.5 h-3.5"
                                                            />
                                                        </button>
                                                    </div>

                                                    // Nested children
                                                    <Show when=move || {
                                                        sub_count > 0 && !is_collapsed.get()
                                                    }>
                                                        <crate::components::subtask_list::SubtaskList
                                                            parent_id=task_id
                                                            parent_sequential_limit=seq_limit
                                                            depth=depth + 1
                                                            project_id=project_id
                                                            on_navigate_to=on_navigate_to
                                                            on_parent_refetch=on_parent_refetch
                                                        />
                                                    </Show>
                                                }
                                            }
                                        </Show>
                                    }
                                })
                                .collect::<Vec<_>>()}

                            // Add subtask form
                            <Show when=move || depth < 5>
                                <SubtaskForm
                                    parent_id=parent_id
                                    project_id=project_id
                                    depth=depth
                                    on_created=on_created
                                />
                            </Show>
                        </div>
                    }.into_any()
                })
            }}
        </Suspense>
    }
}
