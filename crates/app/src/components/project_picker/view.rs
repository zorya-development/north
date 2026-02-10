use leptos::prelude::*;
use north_domain::Project;
use north_ui::Popover;

#[component]
pub fn ProjectPickerView(
    task_id: i64,
    has_project: bool,
    project_title: Option<String>,
    popover_open: ReadSignal<bool>,
    set_popover_open: WriteSignal<bool>,
    projects: Resource<Result<Vec<Project>, ServerFnError>>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
) -> impl IntoView {
    view! {
        <Popover
            open=popover_open
            set_open=set_popover_open
            trigger=Box::new({
                let project_title = project_title.clone();
                move || {
                    if has_project {
                        let display = project_title.clone().unwrap_or_default();
                        view! {
                            <button
                                class="inline-flex items-center gap-1 \
                                       text-text-secondary hover:bg-bg-tertiary \
                                       px-1.5 py-0.5 rounded transition-colors"
                                on:click=move |_| {
                                    set_popover_open.update(|o| *o = !*o);
                                }
                            >
                                <span
                                    class="w-2.5 h-2.5 rounded-full flex-shrink-0 \
                                           bg-text-tertiary"
                                />
                                {display}
                                <span
                                    class="hover:text-text-primary ml-0.5 cursor-pointer"
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        on_clear_project.run(task_id);
                                    }
                                >
                                    "\u{00d7}"
                                </span>
                            </button>
                        }
                        .into_any()
                    } else {
                        view! {
                            <button
                                class="inline-flex items-center gap-1 \
                                       text-text-tertiary hover:text-text-secondary \
                                       hover:bg-bg-tertiary px-1.5 py-0.5 rounded \
                                       transition-colors opacity-0 \
                                       group-hover:opacity-100"
                                on:click=move |_| {
                                    set_popover_open.update(|o| *o = !*o);
                                }
                            >
                                <span
                                    class="w-2.5 h-2.5 rounded-full flex-shrink-0 \
                                           bg-text-tertiary"
                                />
                                "Project"
                            </button>
                        }
                        .into_any()
                    }
                }
            })
        >
            <div class="p-1 w-[200px] max-h-[240px] overflow-y-auto">
                <Suspense fallback=move || {
                    view! {
                        <div class="px-3 py-2 text-xs text-text-tertiary">
                            "Loading..."
                        </div>
                    }
                }>
                    {move || {
                        Suspend::new(async move {
                            match projects.await {
                                Ok(list) => {
                                    if list.is_empty() {
                                        view! {
                                            <div class="px-3 py-2 text-xs \
                                                        text-text-tertiary">
                                                "No projects"
                                            </div>
                                        }
                                        .into_any()
                                    } else {
                                        view! {
                                            <div>
                                                {if has_project {
                                                    Some(view! {
                                                        <button
                                                            class="w-full text-left px-3 \
                                                                   py-1.5 text-xs \
                                                                   text-text-tertiary \
                                                                   hover:bg-bg-tertiary \
                                                                   rounded transition-colors"
                                                            on:click=move |_| {
                                                                set_popover_open.set(false);
                                                                on_clear_project.run(task_id);
                                                            }
                                                        >
                                                            "None"
                                                        </button>
                                                    })
                                                } else {
                                                    None
                                                }}
                                                {list
                                                    .into_iter()
                                                    .map(|p| {
                                                        let pid = p.id;
                                                        let title = p.title.clone();
                                                        let color = p.color.clone();
                                                        view! {
                                                            <button
                                                                class="w-full text-left \
                                                                       px-3 py-1.5 text-sm \
                                                                       text-text-primary \
                                                                       hover:bg-bg-tertiary \
                                                                       rounded \
                                                                       transition-colors \
                                                                       flex items-center \
                                                                       gap-2"
                                                                on:click=move |_| {
                                                                    set_popover_open
                                                                        .set(false);
                                                                    on_set_project
                                                                        .run((task_id, pid));
                                                                }
                                                            >
                                                                <span
                                                                    class="w-2.5 h-2.5 \
                                                                           rounded-full \
                                                                           flex-shrink-0"
                                                                    style=format!(
                                                                        "background-color: {}",
                                                                        color,
                                                                    )
                                                                />
                                                                {title}
                                                            </button>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </div>
                                        }
                                        .into_any()
                                    }
                                }
                                Err(_) => {
                                    view! {
                                        <div class="px-3 py-2 text-xs \
                                                    text-red-500">
                                            "Failed to load"
                                        </div>
                                    }
                                    .into_any()
                                }
                            }
                        })
                    }}
                </Suspense>
            </div>
        </Popover>
    }
}
