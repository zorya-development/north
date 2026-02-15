use leptos::prelude::*;

use crate::components::task_list::TaskList;

#[component]
pub fn ReviewView(
    review_task_ids: Memo<Vec<i64>>,
    reviewed_task_ids: Memo<Vec<i64>>,
    is_loaded: Signal<bool>,
    show_reviewed: ReadSignal<bool>,
    set_show_reviewed: WriteSignal<bool>,
    on_review_all: Callback<()>,
    on_task_click: Callback<i64>,
) -> impl IntoView {
    let empty_reorder_tasks = Memo::new(|_| vec![]);
    let empty_reorder_tasks2 = Memo::new(|_| vec![]);

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <h1 class="text-xl font-semibold text-text-primary">"Review"</h1>
                <button
                    on:click=move |_| on_review_all.run(())
                    class="px-3 py-1.5 text-sm bg-accent text-white rounded \
                           hover:bg-accent-hover transition-colors"
                >
                    "Mark All as Reviewed"
                </button>
            </div>

            <TaskList
                active_task_ids=review_task_ids
                active_tasks_for_reorder=empty_reorder_tasks
                is_loaded=is_loaded
                show_review=true
                show_project=true
                on_reorder=Callback::new(|_| {})
                on_task_click=on_task_click
                empty_message="All tasks are up to date. Nothing to review."
            />

            <div class="border-t border-border pt-4">
                <button
                    on:click=move |_| {
                        set_show_reviewed.update(|v| *v = !*v);
                    }
                    class="text-sm text-text-secondary \
                           hover:text-text-primary transition-colors"
                >
                    {move || {
                        if show_reviewed.get() {
                            "Hide recently reviewed"
                        } else {
                            "Show recently reviewed"
                        }
                    }}
                </button>
                <Show when=move || show_reviewed.get()>
                    <div class="mt-3">
                        <TaskList
                            active_task_ids=reviewed_task_ids
                            active_tasks_for_reorder=empty_reorder_tasks2
                            is_loaded=is_loaded
                            show_review=true
                            show_project=true
                            on_reorder=Callback::new(|_| {})
                            on_task_click=on_task_click
                            empty_message="No recently reviewed tasks."
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}
