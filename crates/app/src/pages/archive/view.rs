use leptos::prelude::*;
use north_dto::Project;

use crate::atoms::{Text, TextColor, TextTag, TextVariant};

#[component]
pub fn ArchiveView(
    archived_projects: Memo<Vec<Project>>,
    is_loaded: Signal<bool>,
    on_unarchive: Callback<i64>,
    on_delete: Callback<i64>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <Text variant=TextVariant::HeadingLg>"Archive"</Text>

            <Show
                when=move || is_loaded.get()
                fallback=|| {
                    view! {
                        <Text variant=TextVariant::BodyMd color=TextColor::Secondary tag=TextTag::P class="py-4">
                            "Loading..."
                        </Text>
                    }
                }
            >
                {move || {
                    let projects = archived_projects.get();
                    if projects.is_empty() {
                        view! {
                            <Text variant=TextVariant::BodyMd color=TextColor::Secondary tag=TextTag::P class="py-8 text-center">
                                "No archived projects."
                            </Text>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="space-y-1">
                                <For
                                    each=move || archived_projects.get()
                                    key=|p| p.id
                                    let:project
                                >
                                    {
                                        let pid = project.id;
                                        let href = format!("/projects/{}", project.id);
                                        view! {
                                            <div class="flex items-center justify-between \
                                                        px-3 py-2 rounded-md bg-bg-secondary">
                                                <a
                                                    href=href
                                                    class="text-sm text-text-primary \
                                                           hover:underline"
                                                >
                                                    {project.title.clone()}
                                                </a>
                                                <div class="flex items-center gap-2">
                                                    <button
                                                        class="text-xs px-2 py-1 rounded \
                                                               bg-bg-tertiary \
                                                               text-text-secondary \
                                                               hover:text-text-primary \
                                                               transition-colors"
                                                        on:click=move |_| {
                                                            on_unarchive.run(pid)
                                                        }
                                                    >
                                                        "Unarchive"
                                                    </button>
                                                    <button
                                                        class="text-xs px-2 py-1 rounded \
                                                               bg-bg-tertiary text-danger \
                                                               hover:text-danger-hover \
                                                               transition-colors"
                                                        on:click=move |_| {
                                                            on_delete.run(pid)
                                                        }
                                                    >
                                                        "Delete"
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }
                                </For>
                            </div>
                        }
                        .into_any()
                    }
                }}
            </Show>
        </div>
    }
}
