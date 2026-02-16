use leptos::prelude::*;
use north_dto::Task;

use crate::atoms::{Text, TextVariant};
use crate::containers::task_list::{CompletedSection, TaskList};

#[component]
pub fn InboxView(
    active_task_ids: Memo<Vec<i64>>,
    completed_task_ids: Memo<Vec<i64>>,
    completed_count: Memo<usize>,
    is_loaded: Signal<bool>,
    on_add_task: Callback<()>,
    on_task_click: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    active_tasks_for_reorder: Memo<Vec<Task>>,
) -> impl IntoView {
    let empty_completed_tasks = Memo::new(|_| vec![]);

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <Text variant=TextVariant::HeadingLg>"Inbox"</Text>
                <button
                    on:click=move |_| on_add_task.run(())
                    class="text-sm text-text-secondary hover:text-accent \
                           transition-colors cursor-pointer"
                >
                    "+" " Add task"
                </button>
            </div>

            <TaskList
                active_task_ids=active_task_ids
                active_tasks_for_reorder=active_tasks_for_reorder
                is_loaded=is_loaded
                draggable=true
                on_reorder=on_reorder
                on_task_click=on_task_click
                empty_message="No tasks in your inbox. Add one above."
            />

            <CompletedSection
                task_ids=completed_task_ids
                tasks_for_reorder=empty_completed_tasks
                count=completed_count
                is_loaded=is_loaded
                on_task_click=on_task_click
            />
        </div>
    }
}
