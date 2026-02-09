use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::icons::{Icon, IconKind};
use crate::server_fns::projects::{archive_project, create_project, get_projects};

#[component]
pub fn Sidebar() -> impl IntoView {
    let projects = Resource::new(|| (), |_| get_projects());
    let (creating, set_creating) = signal(false);
    let (new_title, set_new_title) = signal(String::new());

    let create_action = Action::new(move |title: &String| {
        let title = title.clone();
        create_project(title)
    });

    let archive_action = Action::new(move |id: &i64| {
        let id = *id;
        archive_project(id)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = create_action.value().get() {
            set_creating.set(false);
            set_new_title.set(String::new());
            projects.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = archive_action.value().get() {
            projects.refetch();
        }
    });

    view! {
        <aside class="w-56 bg-bg-secondary flex flex-col h-full">
            <div class="py-4 px-2 flex items-center gap-2">
                <img src="/public/logo.png" alt="North" class="w-10 h-10"/>
                <span class="text-lg font-semibold text-text-primary">"North"</span>
            </div>

            <nav class="flex-1 px-2 space-y-1">
                <NavItem href="/inbox" label="Inbox" icon=IconKind::Inbox/>
                <NavItem href="/today" label="Today" icon=IconKind::Today/>
                <NavItem href="/tasks" label="All Tasks" icon=IconKind::Tasks/>

                <div class="pt-4">
                    <div class="flex items-center justify-between px-3">
                        <span class="text-xs font-medium text-text-secondary \
                                     uppercase tracking-wide">
                            "Projects"
                        </span>
                        <button
                            class="p-0.5 rounded text-text-tertiary \
                                   hover:text-text-secondary \
                                   hover:bg-bg-tertiary transition-colors"
                            on:click=move |_| {
                                set_creating.update(|v| *v = !*v);
                            }
                        >
                            <Icon kind=IconKind::Plus class="w-3.5 h-3.5"/>
                        </button>
                    </div>

                    <Show when=move || creating.get()>
                        <form
                            class="px-1 mt-1"
                            on:submit=move |ev| {
                                ev.prevent_default();
                                let title = new_title.get_untracked();
                                if !title.trim().is_empty() {
                                    create_action.dispatch(title.trim().to_string());
                                }
                            }
                        >
                            <input
                                type="text"
                                class="w-full bg-bg-input border border-border \
                                       rounded px-2 py-1.5 text-sm \
                                       text-text-primary placeholder:text-text-tertiary \
                                       focus:outline-none focus:border-accent"
                                placeholder="Project name"
                                autofocus=true
                                bind:value=(new_title, set_new_title)
                                on:keydown=move |ev| {
                                    if ev.key() == "Escape" {
                                        set_creating.set(false);
                                        set_new_title.set(String::new());
                                    }
                                }
                            />
                        </form>
                    </Show>

                    <Suspense fallback=|| ()>
                        {move || {
                            Suspend::new(async move {
                                match projects.await {
                                    Ok(list) => {
                                        view! {
                                            <div class="mt-1 space-y-0.5">
                                                {list
                                                    .into_iter()
                                                    .map(|p| {
                                                        let pid = p.id;
                                                        let title = p.title.clone();
                                                        view! {
                                                            <ProjectItem
                                                                id=pid
                                                                title=title
                                                                on_archive=move || {
                                                                    archive_action
                                                                        .dispatch(pid);
                                                                }
                                                            />
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </div>
                                        }
                                        .into_any()
                                    }
                                    Err(_) => view! { <div/> }.into_any(),
                                }
                            })
                        }}
                    </Suspense>

                    <div class="mt-1">
                        <NavItem href="/archive" label="Archive" icon=IconKind::Archive/>
                    </div>
                </div>

                <div class="pt-4">
                    <NavItem href="/review" label="Review" icon=IconKind::Review/>
                    <NavItem href="/filter" label="Filters" icon=IconKind::Filter/>
                    <NavItem href="/stats" label="Stats" icon=IconKind::Stats/>
                </div>
            </nav>

            <div class="p-2 border-t border-border">
                <NavItem href="/settings" label="Settings" icon=IconKind::Settings/>
            </div>
        </aside>
    }
}

#[component]
fn ProjectItem(
    id: i64,
    title: String,
    on_archive: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    let (hover, set_hover) = signal(false);
    let on_archive = std::sync::Arc::new(on_archive);
    let href = format!("/projects/{id}");
    let location = use_location();
    let href_cmp = href.clone();

    let class = move || {
        let base = "group flex items-center gap-2 px-3 py-1.5 rounded-md \
                    text-sm text-text-primary hover:bg-bg-tertiary \
                    select-none transition-colors";
        if location.pathname.get() == href_cmp {
            format!("{base} bg-bg-tertiary font-medium")
        } else {
            base.to_string()
        }
    };

    view! {
        <a
            href=href
            class=class
            on:mouseenter=move |_| set_hover.set(true)
            on:mouseleave=move |_| set_hover.set(false)
        >
            <Icon kind=IconKind::Folder class="w-4 h-4 flex-shrink-0 \
                                               text-text-tertiary"/>
            <span class="flex-1 truncate">{title}</span>
            <Show when=move || hover.get()>
                {
                    let on_archive = on_archive.clone();
                    view! {
                <button
                    class="p-0.5 rounded text-text-tertiary \
                           hover:text-text-secondary transition-colors"
                    title="Archive project"
                    on:click=move |ev| {
                        ev.prevent_default();
                        ev.stop_propagation();
                        on_archive();
                    }
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="w-3 h-3"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <polyline points="21 8 21 21 3 21 3 8"/>
                        <rect x="1" y="3" width="22" height="5"/>
                        <line x1="10" y1="12" x2="14" y2="12"/>
                    </svg>
                </button>
                    }
                }
            </Show>
        </a>
    }
}

#[component]
fn NavItem(href: &'static str, label: &'static str, icon: IconKind) -> impl IntoView {
    let location = use_location();

    let is_active = move || location.pathname.get() == href;

    let class = move || {
        let base = "flex items-center gap-2 px-3 py-2 rounded-md text-sm \
                    text-text-primary hover:bg-bg-tertiary transition-colors";
        if is_active() {
            format!("{base} bg-bg-tertiary font-medium")
        } else {
            base.to_string()
        }
    };

    view! {
        <a href=href class=class>
            <Icon kind=icon class="w-4 h-4 flex-shrink-0"/>
            {label}
        </a>
    }
}
