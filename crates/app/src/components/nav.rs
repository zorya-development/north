use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::icons::{Icon, IconKind};
use crate::server_fns::projects::{
    archive_project, create_project, get_projects, update_project_details,
};

const PRESET_COLORS: &[&str] = &[
    "#6b7280", "#ef4444", "#f97316", "#eab308", "#22c55e",
    "#06b6d4", "#3b82f6", "#8b5cf6", "#ec4899",
];

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

    let edit_action =
        Action::new(move |input: &(i64, String, String)| {
            let (id, title, color) = input.clone();
            update_project_details(id, title, color)
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

    Effect::new(move || {
        if let Some(Ok(_)) = edit_action.value().get() {
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
                                                        let color = p.color.clone();
                                                        view! {
                                                            <ProjectItem
                                                                id=pid
                                                                title=title
                                                                color=color
                                                                on_archive=move || {
                                                                    archive_action
                                                                        .dispatch(pid);
                                                                }
                                                                on_edit=move |id, t, c| {
                                                                    edit_action
                                                                        .dispatch((id, t, c));
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
    color: String,
    on_archive: impl Fn() + Send + Sync + 'static,
    on_edit: impl Fn(i64, String, String) + Send + Sync + 'static,
) -> impl IntoView {
    let (hover, set_hover) = signal(false);
    let (editing, set_editing) = signal(false);
    let (edit_title, set_edit_title) = signal(title.clone());
    let (edit_color, set_edit_color) = signal(color.clone());
    let on_archive = std::sync::Arc::new(on_archive);
    let on_edit = std::sync::Arc::new(on_edit);
    let href = format!("/projects/{id}");
    let location = use_location();
    let href_cmp = href.clone();

    let class = Memo::new(move |_| {
        let base = "group flex items-center gap-2 px-3 py-1.5 rounded-md \
                    text-sm text-text-primary hover:bg-bg-tertiary \
                    select-none transition-colors";
        if location.pathname.get() == href_cmp {
            format!("{base} bg-bg-tertiary font-medium")
        } else {
            base.to_string()
        }
    });

    let color_dot = color.clone();

    view! {
        <Show
            when=move || editing.get()
            fallback={
                let href = href.clone();
                let color_dot = color_dot.clone();
                let on_archive = on_archive.clone();
                move || {
                    let on_archive = on_archive.clone();
                    let color_dot = color_dot.clone();
                    view! {
                        <a
                            href=href.clone()
                            class=class
                            on:mouseenter=move |_| set_hover.set(true)
                            on:mouseleave=move |_| set_hover.set(false)
                        >
                            <span
                                class="w-3 h-3 rounded-full flex-shrink-0"
                                style=format!(
                                    "background-color: {}",
                                    color_dot,
                                )
                            />
                            <span class="flex-1 truncate">{title.clone()}</span>
                            <Show when=move || hover.get()>
                                {
                                    let on_archive = on_archive.clone();
                                    view! {
                                        <button
                                            class="p-0.5 rounded text-text-tertiary \
                                                   hover:text-text-secondary \
                                                   transition-colors"
                                            title="Edit project"
                                            on:click=move |ev| {
                                                ev.prevent_default();
                                                ev.stop_propagation();
                                                set_editing.set(true);
                                            }
                                        >
                                            <Icon
                                                kind=IconKind::Edit
                                                class="w-3 h-3"
                                            />
                                        </button>
                                        <button
                                            class="p-0.5 rounded text-text-tertiary \
                                                   hover:text-text-secondary \
                                                   transition-colors"
                                            title="Archive project"
                                            on:click=move |ev| {
                                                ev.prevent_default();
                                                ev.stop_propagation();
                                                on_archive();
                                            }
                                        >
                                            <Icon
                                                kind=IconKind::Archive
                                                class="w-3 h-3"
                                            />
                                        </button>
                                    }
                                }
                            </Show>
                        </a>
                    }
                }
            }
        >
            {
                let on_edit = on_edit.clone();
                move || {
                    let on_edit = on_edit.clone();
                    view! {
                        <div class="px-1 py-1">
                            <input
                                type="text"
                                class="w-full bg-bg-input border border-border \
                                       rounded px-2 py-1 text-sm \
                                       text-text-primary \
                                       focus:outline-none focus:border-accent"
                                prop:value=move || edit_title.get()
                                on:input=move |ev| {
                                    set_edit_title
                                        .set(event_target_value(&ev));
                                }
                                on:keydown=move |ev| {
                                    if ev.key() == "Escape" {
                                        set_editing.set(false);
                                    }
                                }
                            />
                            <div class="flex gap-1 mt-1.5 px-0.5">
                                {PRESET_COLORS
                                    .iter()
                                    .map(|c| {
                                        let c = c.to_string();
                                        let c2 = c.clone();
                                        let c3 = c.clone();
                                        view! {
                                            <button
                                                class="w-5 h-5 rounded-full \
                                                       border-2 transition-colors"
                                                style=move || {
                                                    let border =
                                                        if edit_color.get() == c3 {
                                                            "border-color: white"
                                                        } else {
                                                            "border-color: transparent"
                                                        };
                                                    format!(
                                                        "background-color: {}; {}",
                                                        c3, border,
                                                    )
                                                }
                                                on:click={
                                                    let c = c2.clone();
                                                    move |_| {
                                                        set_edit_color
                                                            .set(c.clone());
                                                    }
                                                }
                                            />
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                            <div class="flex gap-1 mt-1.5">
                                <button
                                    class="px-2 py-0.5 text-xs \
                                           text-text-secondary \
                                           hover:text-text-primary \
                                           transition-colors"
                                    on:click=move |_| {
                                        set_editing.set(false);
                                    }
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="px-2 py-0.5 text-xs bg-accent \
                                           text-white rounded \
                                           hover:bg-accent-hover \
                                           transition-colors"
                                    on:click=move |_| {
                                        let t = edit_title
                                            .get_untracked()
                                            .trim()
                                            .to_string();
                                        if !t.is_empty() {
                                            on_edit(
                                                id,
                                                t,
                                                edit_color.get_untracked(),
                                            );
                                            set_editing.set(false);
                                        }
                                    }
                                >
                                    "Save"
                                </button>
                            </div>
                        </div>
                    }
                }
            }
        </Show>
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
