use leptos::prelude::*;
use north_dto::Task;

use super::container::TaskList;

#[component]
pub fn CompletedSection(
    task_ids: Memo<Vec<i64>>,
    tasks_for_reorder: Memo<Vec<Task>>,
    count: Memo<usize>,
    is_loaded: Signal<bool>,
    #[prop(default = true)] show_project: bool,
    on_task_click: Callback<i64>,
) -> impl IntoView {
    let (showing, set_showing) = signal(false);

    view! {
        {move || {
            let count = count.get();
            if count == 0 {
                return None;
            }
            Some(view! {
                <div class="mt-4 border-t border-(--border-muted) pt-3">
                    <button
                        class="text-xs text-text-secondary \
                               hover:text-text-primary transition-colors"
                        on:click=move |_| set_showing.update(|v| *v = !*v)
                    >
                        {move || {
                            if showing.get() {
                                format!("Hide completed ({count})")
                            } else {
                                format!("Show completed ({count})")
                            }
                        }}
                    </button>
                    <Show when=move || showing.get()>
                        <div class="mt-2 opacity-60">
                            <TaskList
                                active_task_ids=task_ids
                                active_tasks_for_reorder=tasks_for_reorder
                                show_project=show_project
                                compact=true
                                is_loaded=is_loaded
                                on_reorder=Callback::new(|_| {})
                                on_task_click=on_task_click
                            />
                        </div>
                    </Show>
                </div>
            })
        }}
    }
}
