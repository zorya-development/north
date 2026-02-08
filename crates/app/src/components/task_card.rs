use leptos::prelude::*;
use north_domain::TaskWithMeta;

#[component]
pub fn TaskCard(task: TaskWithMeta) -> impl IntoView {
    let title = task.task.title.clone();
    let project_title = task.project_title.clone();
    let due_date = task.task.due_date;
    let tags = task.tags.clone();

    view! {
        <div class="bg-peach-100 rounded-lg p-3 border border-peach-200 \
                    hover:border-teal-500 hover:shadow-sm transition-colors">
            <div class="flex items-center gap-2">
                <button
                    class="w-4 h-4 rounded-full border-2 border-teal-500 \
                           hover:bg-teal-500 transition-colors flex-shrink-0"
                    aria-label="Complete task"
                />
                <span class="text-sm font-medium text-teal-950">{title}</span>
            </div>

            {(!project_title.is_none() || due_date.is_some() || !tags.is_empty())
                .then(move || {
                    let project_title = project_title.clone();
                    let tags = tags.clone();
                    view! {
                        <div class="mt-1 ml-6 flex items-center gap-2 text-xs text-sage-400">
                            {project_title
                                .map(|p| {
                                    view! {
                                        <span class="text-teal-700">{p}</span>
                                    }
                                })}
                            {due_date
                                .map(|d| {
                                    view! {
                                        <span>{format!("Due {d}")}</span>
                                    }
                                })}
                            {tags
                                .into_iter()
                                .map(|tag| {
                                    view! {
                                        <span class="bg-peach-200 text-teal-700 text-xs \
                                                     px-2 py-0.5 rounded-full">
                                            {tag}
                                        </span>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }
                })}
        </div>
    }
}
