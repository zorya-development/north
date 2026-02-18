use leptos::prelude::*;

use crate::atoms::{Text, TextVariant};
use crate::containers::traversable_task_list::TraversableTaskList;

#[component]
pub fn InboxView(
    root_task_ids: Memo<Vec<i64>>,
    show_completed: RwSignal<bool>,
    completed_count: Memo<usize>,
    is_loaded: Signal<bool>,
    on_add_task: Callback<()>,
    on_task_click: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <Text variant=TextVariant::HeadingLg>"Inbox"</Text>
                <div class="flex items-center gap-3">
                    {move || {
                        let count = completed_count.get();
                        if count > 0 {
                            Some(
                                view! {
                                    <button
                                        on:click=move |_| {
                                            show_completed.update(|v| *v = !*v)
                                        }
                                        class="text-xs text-text-secondary \
                                               hover:text-text-primary \
                                               transition-colors cursor-pointer"
                                    >
                                        {move || {
                                            if show_completed.get() {
                                                format!(
                                                    "Hide completed ({count})",
                                                )
                                            } else {
                                                format!(
                                                    "Show completed ({count})",
                                                )
                                            }
                                        }}
                                    </button>
                                },
                            )
                        } else {
                            None
                        }
                    }}
                    <button
                        on:click=move |_| on_add_task.run(())
                        class="text-sm text-text-secondary hover:text-accent \
                               transition-colors cursor-pointer"
                    >
                        "+" " Add task"
                    </button>
                </div>
            </div>

            <TraversableTaskList
                root_task_ids=root_task_ids
                show_completed=show_completed
                is_loaded=is_loaded
                on_reorder=on_reorder
                on_task_click=on_task_click
                empty_message="No tasks in your inbox. Add one above."
            />
        </div>
    }
}
