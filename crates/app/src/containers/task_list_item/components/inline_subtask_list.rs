use leptos::prelude::*;
use north_stores::{use_app_store, IdFilter, TaskStoreFilter};

use crate::containers::inline_task_input::InlineTaskInput;
use crate::containers::task_list::ExtraVisibleIds;
use crate::containers::task_list_item::TaskListItem;

#[component]
pub fn InlineSubtaskList(
    parent_id: i64,
    sequential_limit: i16,
    #[prop(default = true)] show_project: bool,
    #[prop(default = false)] draggable: bool,
    #[prop(default = 1)] depth: u8,
    #[prop(optional)] on_click: Option<Callback<i64>>,
    #[prop(default = "")] class: &'static str,
    #[prop(default = "")] add_btn_class: &'static str,
) -> impl IntoView {
    let app_store = use_app_store();
    let (show_non_actionable, set_show_non_actionable) = signal(false);
    let (show_completed, set_show_completed) = signal(false);
    let (show_inline_input, set_show_inline_input) = signal(false);
    let input_value = RwSignal::new(String::new());
    let extra_visible_ids = expect_context::<ExtraVisibleIds>().0;
    let limit = sequential_limit as usize;

    let all_subtasks = app_store.tasks.filtered(TaskStoreFilter {
        parent_id: IdFilter::Is(parent_id),
        ..Default::default()
    });

    let uncompleted = app_store.tasks.filtered(TaskStoreFilter {
        parent_id: IdFilter::Is(parent_id),
        is_completed: Some(false),
        ..Default::default()
    });

    let completed = app_store.tasks.filtered(TaskStoreFilter {
        parent_id: IdFilter::Is(parent_id),
        is_completed: Some(true),
        ..Default::default()
    });

    let visible_ids = Memo::new(move |_| {
        let tasks = uncompleted.get();
        let total = tasks.len();
        let extra = extra_visible_ids.get();
        let mut ids: Vec<i64> = if !show_non_actionable.get() && limit > 0 && total > limit {
            tasks.iter().take(limit).map(|t| t.id).collect()
        } else {
            tasks.iter().map(|t| t.id).collect()
        };
        // Always include extra visible IDs (never hide them)
        for id in &extra {
            if !ids.contains(id) && tasks.iter().any(|t| t.id == *id) {
                ids.push(*id);
            }
        }
        if show_completed.get() {
            ids.extend(completed.get().iter().map(|t| t.id));
        }
        ids
    });

    let non_actionable_count = Memo::new(move |_| {
        let total = uncompleted.get().len();
        if limit > 0 && total > limit {
            total - limit
        } else {
            0
        }
    });

    let completed_count = Memo::new(move |_| completed.get().len());
    let total_count = Memo::new(move |_| all_subtasks.get().len());

    view! {
        <div class=format!("{class}")>
            {move || {
                visible_ids
                    .get()
                    .into_iter()
                    .map(|id| {
                        view! {
                            <TaskListItem
                                task_id=id
                                show_project=show_project
                                draggable=draggable
                                depth=depth
                                on_click=on_click
                                    .unwrap_or(Callback::new(|_| {}))
                            />
                        }
                    })
                    .collect_view()
            }}
            // Inline task input
            <Show when=move || show_inline_input.get()>
                <InlineTaskInput
                    parent_id=parent_id
                    value=input_value
                    on_created=Callback::new(move |id| {
                        extra_visible_ids.update(|ids| {
                            if !ids.contains(&id) {
                                ids.push(id);
                            }
                        });
                    })
                    on_close=Callback::new(move |()| {
                        set_show_inline_input.set(false);
                    })
                    class=add_btn_class
                />
            </Show>
            // Add subtask button
            <Show when=move || !show_inline_input.get()>
                <button
                    class=format!(
                        "{add_btn_class} my-3 text-xs text-accent \
                         hover:text-accent-hover \
                         hover:underline cursor-pointer \
                         transition-colors"
                    )
                    on:click=move |_| {
                        set_show_inline_input.set(true);
                    }
                >
                    "+ Add subtask"
                </button>
            </Show>
            // Toggle bar
            <Show when=move || {
                non_actionable_count.get() > 0usize
                    || completed_count.get() > 0usize
            }>
                <div class="ml-12 py-1 flex items-center gap-2 text-xs">
                    // Show N More / Hide Non Actionable
                    <Show when=move || {
                        non_actionable_count.get() > 0usize
                    }>
                        <button
                            class="text-accent hover:text-accent-hover \
                                   hover:underline cursor-pointer \
                                   transition-colors"
                            on:click=move |_| {
                                set_show_non_actionable
                                    .update(|v| *v = !*v);
                            }
                        >
                            {move || {
                                if show_non_actionable.get() {
                                    "Hide Non Actionable".to_string()
                                } else {
                                    format!(
                                        "Show {} More",
                                        non_actionable_count.get(),
                                    )
                                }
                            }}
                        </button>
                    </Show>
                    // Show / Hide Completed
                    <Show when=move || {
                        completed_count.get() > 0usize
                    }>
                        <button
                            class="text-accent hover:text-accent-hover \
                                   hover:underline cursor-pointer \
                                   transition-colors"
                            on:click=move |_| {
                                set_show_completed
                                    .update(|v| *v = !*v);
                            }
                        >
                            {move || {
                                if show_completed.get() {
                                    "Hide Completed".to_string()
                                } else {
                                    format!(
                                        "Show Completed ({})",
                                        completed_count.get(),
                                    )
                                }
                            }}
                        </button>
                    </Show>
                    // Total
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
