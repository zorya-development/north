use leptos::prelude::*;

use crate::components::task_detail_modal::{TaskDetailContext, TaskDetailModal};
use crate::components::task_list::TaskList;
use crate::server_fns::tasks::{
    get_recently_reviewed_tasks, get_review_tasks, review_all_tasks,
};
use crate::stores::task_store::TaskStore;

#[component]
pub fn ReviewPage() -> impl IntoView {
    let open_task_id = RwSignal::new(None::<i64>);
    provide_context(TaskDetailContext { open_task_id });

    let review_tasks = Resource::new(|| (), |_| get_review_tasks());
    let store = TaskStore::new(review_tasks);

    let review_all_action = Action::new(|_: &()| review_all_tasks());

    Effect::new(move || {
        if let Some(Ok(_)) = review_all_action.value().get() {
            review_tasks.refetch();
        }
    });

    let (show_reviewed, set_show_reviewed) = signal(false);
    let reviewed_tasks = Resource::new(
        move || show_reviewed.get(),
        move |show| async move {
            if show {
                get_recently_reviewed_tasks().await
            } else {
                Ok(vec![])
            }
        },
    );
    let reviewed_store = TaskStore::new(reviewed_tasks);

    let modal_store = store.clone();

    let task_ids = Signal::derive(move || {
        review_tasks
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .iter()
            .map(|t| t.task.id)
            .collect::<Vec<_>>()
    });

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-semibold tracking-tight text-text-primary">"Review"</h1>
                <button
                    on:click=move |_| { review_all_action.dispatch(()); }
                    class="px-3 py-1.5 text-sm bg-accent text-on-accent rounded \
                           hover:bg-accent-hover transition-colors"
                >
                    "Mark All as Reviewed"
                </button>
            </div>
            <TaskList
                resource=review_tasks
                store=store
                show_review=true
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
                    {
                        let reviewed_store = reviewed_store.clone();
                        view! {
                            <div class="mt-3">
                                <TaskList
                                    resource=reviewed_tasks
                                    store=reviewed_store
                                    show_review=true
                                    empty_message="No recently reviewed tasks."
                                />
                            </div>
                        }
                    }
                </Show>
            </div>
            <TaskDetailModal task_ids=task_ids task_store=modal_store/>
        </div>
    }
}
