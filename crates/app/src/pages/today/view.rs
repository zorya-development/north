use leptos::prelude::*;
use north_dto::Task;

use super::controller::GroupedTasks;
use crate::components::task_list::{CompletedSection, TaskList};
use crate::containers::task_inline_form::TaskInlineForm;

#[component]
pub fn TodayView(
    grouped_task_ids: Memo<GroupedTasks>,
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
                <h1 class="text-2xl font-semibold tracking-tight text-text-primary">"Today"</h1>
                <button
                    on:click=move |_| set_form_open.set(!is_form_open.get_untracked())
                    class="text-sm text-text-secondary hover:text-accent \
                           transition-colors cursor-pointer"
                >
                    "+" " Add task"
                </button>
            </div>

            <Show when=move || is_form_open.get()>
                <TaskInlineForm on_done=Callback::new(move |()| {
                    set_form_open.set(false)
                })/>
            </Show>

            {move || {
                if !is_loaded.get() {
                    return view! {
                        <div class="text-sm text-text-secondary py-4">"Loading tasks..."</div>
                    }.into_any();
                }

                let groups = grouped_task_ids.get();
                if groups.is_empty() {
                    return view! {
                        <div class="text-sm text-text-secondary py-8 text-center">
                            "No tasks scheduled for today."
                        </div>
                    }.into_any();
                }

                view! {
                    <div class="space-y-4">
                        <For
                            each=move || grouped_task_ids.get()
                            key=|(label, _)| label.clone()
                            let:group
                        >
                            {
                                let (label, ids) = group;
                                let ids = Memo::new(move |_| ids.clone());
                                view! {
                                    <div>
                                        <h2 class="text-xs font-medium text-text-secondary \
                                                    uppercase tracking-wide px-3 pb-1">
                                            {label}
                                        </h2>
                                        <TaskList
                                            active_task_ids=ids
                                            active_tasks_for_reorder=active_tasks_for_reorder
                                            is_loaded=is_loaded
                                            show_project=false
                                            draggable=false
                                            on_reorder=on_reorder
                                            on_task_click=on_task_click
                                        />
                                    </div>
                                }
                            }
                        </For>
                    </div>
                }.into_any()
            }}

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
