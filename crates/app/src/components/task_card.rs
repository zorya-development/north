use leptos::prelude::*;
use north_domain::TaskWithMeta;

#[component]
pub fn TaskCard(task: TaskWithMeta) -> impl IntoView {
    let title = task.task.title.clone();
    let project_title = task.project_title.clone();
    let due_date = task.task.due_date;
    let tags = task.tags.clone();

    view! {
        <div class="border-b border-border px-3 py-2 hover:bg-bg-tertiary transition-colors">
            <div class="flex items-center gap-2">
                <button
                    class="w-4 h-4 rounded-full border-2 border-text-secondary \
                           hover:border-accent hover:bg-accent transition-colors \
                           flex-shrink-0"
                    aria-label="Complete task"
                />
                <span class="text-sm text-text-primary">{title}</span>
            </div>

            {(!project_title.is_none() || due_date.is_some() || !tags.is_empty())
                .then(move || {
                    let project_title = project_title.clone();
                    let tags = tags.clone();
                    view! {
                        <div class="mt-0.5 ml-6 flex items-center gap-2 text-xs \
                                    text-text-tertiary">
                            {project_title
                                .map(|p| {
                                    view! {
                                        <span class="text-text-secondary">{p}</span>
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
                                        <span class="bg-bg-tertiary text-text-secondary \
                                                     text-xs px-2 py-0.5 rounded-full">
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
