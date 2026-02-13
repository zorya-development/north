use leptos::prelude::*;

use crate::server_fns::projects::{delete_project, get_archived_projects, unarchive_project};

#[component]
pub fn ArchivePage() -> impl IntoView {
    let archived = Resource::new(|| (), |_| get_archived_projects());

    let unarchive_action = Action::new(move |id: &i64| {
        let id = *id;
        unarchive_project(id)
    });

    let delete_action = Action::new(move |id: &i64| {
        let id = *id;
        delete_project(id)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = unarchive_action.value().get() {
            archived.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = delete_action.value().get() {
            archived.refetch();
        }
    });

    view! {
        <div class="space-y-4">
            <h1 class="text-2xl font-semibold tracking-tight text-text-primary">"Archive"</h1>
            <Suspense fallback=move || {
                view! {
                    <div class="text-sm text-text-secondary py-4">
                        "Loading..."
                    </div>
                }
            }>
                {move || {
                    Suspend::new(async move {
                        match archived.await {
                            Ok(projects) => {
                                if projects.is_empty() {
                                    view! {
                                        <div class="text-sm text-text-secondary \
                                                    py-8 text-center">
                                            "No archived projects."
                                        </div>
                                    }
                                    .into_any()
                                } else {
                                    view! {
                                        <div class="space-y-1">
                                            {projects
                                                .into_iter()
                                                .map(|p| {
                                                    let pid = p.id;
                                                    let href = format!("/projects/{}", p.id);
                                                    view! {
                                                        <div class="flex items-center \
                                                                    justify-between \
                                                                    px-3 py-2 rounded-md \
                                                                    bg-bg-secondary">
                                                            <a
                                                                href=href
                                                                class="text-sm \
                                                                       text-text-primary \
                                                                       hover:underline"
                                                            >
                                                                {p.title}
                                                            </a>
                                                            <div class="flex items-center \
                                                                        gap-2">
                                                                <button
                                                                    class="text-xs px-2 py-1 \
                                                                           rounded \
                                                                           bg-bg-tertiary \
                                                                           text-text-secondary \
                                                                           hover:text-text-primary \
                                                                           transition-colors"
                                                                    on:click=move |_| {
                                                                        unarchive_action
                                                                            .dispatch(pid);
                                                                    }
                                                                >
                                                                    "Unarchive"
                                                                </button>
                                                                <button
                                                                    class="text-xs px-2 py-1 \
                                                                           rounded \
                                                                           bg-bg-tertiary \
                                                                           text-danger \
                                                                           hover:text-danger-hover \
                                                                           transition-colors"
                                                                    on:click=move |_| {
                                                                        delete_action
                                                                            .dispatch(pid);
                                                                    }
                                                                >
                                                                    "Delete"
                                                                </button>
                                                            </div>
                                                        </div>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    }
                                    .into_any()
                                }
                            }
                            Err(e) => {
                                view! {
                                    <div class="text-sm text-danger py-4">
                                        {format!("Failed to load: {e}")}
                                    </div>
                                }
                                .into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
