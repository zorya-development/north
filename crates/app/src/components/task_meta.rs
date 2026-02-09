use leptos::prelude::*;

use crate::components::date_picker::DateTimePicker;
use crate::components::project_picker::ProjectPicker;

#[component]
pub fn TaskMeta(
    task_id: i64,
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    project_id: Option<i64>,
    project_title: Option<String>,
    due_date: Option<chrono::NaiveDate>,
    is_completed: ReadSignal<bool>,
    tags: Vec<String>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    #[prop(default = None)] reviewed_at: Option<chrono::NaiveDate>,
    #[prop(default = false)] show_review: bool,
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
            <ProjectPicker
                task_id=task_id
                project_id=project_id
                project_title=project_title
                on_set_project=on_set_project
                on_clear_project=on_clear_project
            />
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
            {if show_review {
                Some(view! {
                    <span class="ml-auto whitespace-nowrap">
                        {match reviewed_at {
                            Some(d) => format!("Reviewed {d}"),
                            None => "Never reviewed".to_string(),
                        }}
                    </span>
                })
            } else {
                None
            }}
        </div>
    }
}
