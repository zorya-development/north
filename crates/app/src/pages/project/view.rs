use leptos::prelude::*;
use north_dto::{Project, Task};

use crate::atoms::{Text, TextVariant};
use crate::components::task_list::{CompletedSection, TaskList};
use crate::containers::task_inline_form::TaskInlineForm;

#[component]
pub fn ProjectView(
    project: Memo<Option<Project>>,
    project_id: Signal<i64>,
    active_task_ids: Memo<Vec<i64>>,
    completed_task_ids: Memo<Vec<i64>>,
    completed_count: Memo<usize>,
    is_loaded: Signal<bool>,
    is_form_open: ReadSignal<bool>,
    set_form_open: WriteSignal<bool>,
    on_task_click: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    active_tasks_for_reorder: Memo<Vec<Task>>,
) -> impl IntoView {
    let empty_completed_tasks = Memo::new(|_| vec![]);

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <Text variant=TextVariant::HeadingLg>
                    {move || {
                        project
                            .get()
                            .map(|p| p.title)
                            .unwrap_or_else(|| "Project".to_string())
                    }}
                </Text>
                <button
                    on:click=move |_| set_form_open.set(!is_form_open.get_untracked())
                    class="text-sm text-text-secondary hover:text-accent \
                           transition-colors cursor-pointer"
                >
                    "+" " Add task"
                </button>
            </div>

            <Show when=move || is_form_open.get()>
                <TaskInlineForm
                    project_id=project_id.get_untracked()
                    on_done=Callback::new(move |()| {
                        set_form_open.set(false)
                    })
                />
            </Show>

            <TaskList
                active_task_ids=active_task_ids
                active_tasks_for_reorder=active_tasks_for_reorder
                is_loaded=is_loaded
                show_project=false
                draggable=true
                on_reorder=on_reorder
                on_task_click=on_task_click
                empty_message="No tasks in this project. Add one above."
            />

            <CompletedSection
                task_ids=completed_task_ids
                tasks_for_reorder=empty_completed_tasks
                count=completed_count
                is_loaded=is_loaded
                show_project=false
                on_task_click=on_task_click
            />
        </div>
    }
}
