use leptos::prelude::*;

use crate::components::task_form::InlineTaskForm;
use crate::components::task_list::TaskList;
use crate::server_fns::tasks::{create_task, get_review_tasks, review_all_tasks};
use crate::stores::task_store::TaskStore;

#[component]
pub fn ReviewPage() -> impl IntoView {
    let review_tasks = Resource::new(|| (), |_| get_review_tasks());
    let store = TaskStore::new(review_tasks);

    let create_action = Action::new(|input: &(String, Option<String>)| {
        let (title, body) = input.clone();
        create_task(title, body)
    });

    let review_all_action = Action::new(|_: &()| review_all_tasks());

    Effect::new(move || {
        if let Some(Ok(_)) = create_action.value().get() {
            review_tasks.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = review_all_action.value().get() {
            review_tasks.refetch();
        }
    });

    let on_create = move |title: String, body: Option<String>| {
        create_action.dispatch((title, body));
    };

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <h1 class="text-xl font-semibold text-text-primary">"Review"</h1>
                <button
                    on:click=move |_| { review_all_action.dispatch(()); }
                    class="px-3 py-1.5 text-sm bg-accent text-white rounded \
                           hover:bg-accent-hover transition-colors"
                >
                    "Mark All as Reviewed"
                </button>
            </div>
            <InlineTaskForm on_submit=on_create/>
            <TaskList
                resource=review_tasks
                store=store
                show_review=true
                empty_message="All tasks are up to date. Nothing to review."
            />
        </div>
    }
}
