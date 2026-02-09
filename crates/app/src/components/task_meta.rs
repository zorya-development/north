use leptos::prelude::*;

use crate::components::date_picker::DateTimePicker;

#[component]
pub fn TaskMeta(
    task_id: i64,
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    project_title: Option<String>,
    due_date: Option<chrono::NaiveDate>,
    is_completed: ReadSignal<bool>,
    tags: Vec<String>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
) -> impl IntoView {
    view! {
        <div class="mt-0.5 ml-6 flex items-center gap-2 text-xs \
                    text-text-tertiary">
            <DateTimePicker
                task_id=task_id
                start_at=start_at
                on_set_start_at=on_set_start_at
                on_clear_start_at=on_clear_start_at
            />
            {project_title.map(|p| {
                view! {
                    <span class="text-text-secondary">{p}</span>
                }
            })}
            {due_date.map(|d| {
                view! {
                    <span>{format!("Due {d}")}</span>
                }
            })}
            <Show when=move || is_completed.get()>
                <span>"Completed"</span>
            </Show>
            {tags
                .into_iter()
                .map(|tag| {
                    view! {
                        <span class="bg-bg-tertiary text-text-secondary \
                                     text-xs px-2 py-0.5 rounded-full">
                            {tag}
                        </span>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
