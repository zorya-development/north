use leptos::prelude::*;
use north_dto::Task;

use super::controller::GroupedTasks;
use crate::atoms::{Text, TextColor, TextTag, TextVariant};
use crate::containers::task_list::{CompletedSection, TaskList};

#[component]
pub fn TodayView(
    grouped_task_ids: Memo<GroupedTasks>,
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
                <Text variant=TextVariant::HeadingLg>"Today"</Text>
                <button
                    on:click=move |_| on_add_task.run(())
                    class="text-sm text-text-secondary hover:text-accent \
                           transition-colors cursor-pointer"
                >
                    "+" " Add task"
                </button>
            </div>

            {move || {
                if !is_loaded.get() {
                    return view! {
                        <Text variant=TextVariant::BodyMd color=TextColor::Secondary tag=TextTag::P class="py-4">"Loading tasks..."</Text>
                    }.into_any();
                }

                let groups = grouped_task_ids.get();
                if groups.is_empty() {
                    return view! {
                        <Text variant=TextVariant::BodyMd color=TextColor::Secondary tag=TextTag::P class="py-8 text-center">
                            "No tasks scheduled for today."
                        </Text>
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
                                        <Text variant=TextVariant::LabelMd color=TextColor::Secondary tag=TextTag::H2 class="px-3 pb-1">
                                            {label}
                                        </Text>
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
