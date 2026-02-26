use leptos::prelude::*;
use north_dto::Project;

use crate::atoms::{Text, TextColor, TextTag, TextVariant};
use north_ui::{Icon, IconKind, Popover};

#[component]
pub fn ProjectPickerView(
    task_id: i64,
    has_project: bool,
    project_title: Option<String>,
    popover_open: ReadSignal<bool>,
    set_popover_open: WriteSignal<bool>,
    projects: Memo<Vec<Project>>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    #[prop(default = false)] icon_only: bool,
    #[prop(default = false)] always_visible: bool,
) -> impl IntoView {
    view! {
        <Popover
            open=popover_open
            set_open=set_popover_open
            trigger=Box::new({
                let project_title = project_title.clone();
                move || {
                    if icon_only {
                        return view! {
                            <button
                                class="p-1 rounded hover:bg-bg-input \
                                       text-text-tertiary \
                                       hover:text-text-secondary \
                                       transition-colors"
                                on:click=move |_| {
                                    set_popover_open
                                        .update(|o| *o = !*o);
                                }
                                aria-label="Set project"
                            >
                                <Icon
                                    kind=IconKind::Folder
                                    class="w-4 h-4"
                                />
                            </button>
                        }
                        .into_any();
                    }
                    if has_project {
                        let display = project_title.clone().unwrap_or_default();
                        view! {
                            <button
                                class="inline-flex items-center gap-1 \
                                       text-text-secondary hover:bg-bg-tertiary \
                                       px-1.5 py-0.5 rounded transition-colors \
                                       cursor-pointer select-none"
                                data-testid="project-picker-trigger"
                                on:click=move |_| {
                                    set_popover_open.update(|o| *o = !*o);
                                }
                            >
                                <Icon
                                    kind=IconKind::Folder
                                    class="w-3.5 h-3.5 text-text-tertiary \
                                           flex-shrink-0"
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
                        {
                            let vis_class = if always_visible {
                                "inline-flex items-center gap-1 \
                                 text-text-tertiary hover:text-text-secondary \
                                 hover:bg-bg-tertiary px-1.5 py-0.5 rounded \
                                 transition-colors cursor-pointer select-none"
                            } else {
                                "items-center gap-1 \
                                 text-text-tertiary hover:text-text-secondary \
                                 hover:bg-bg-tertiary px-1.5 py-0.5 rounded \
                                 transition-colors cursor-pointer select-none \
                                 hidden group-hover:inline-flex"
                            };
                            view! {
                                <button
                                    class=vis_class
                                    data-testid="project-picker-trigger"
                                    on:click=move |_| {
                                        set_popover_open.update(|o| *o = !*o);
                                    }
                                >
                                    <Icon
                                        kind=IconKind::Inbox
                                        class="w-3.5 h-3.5"
                                    />
                                    "Inbox"
                                </button>
                            }
                            .into_any()
                        }
                    }
                }
            })
        >
            <div class="p-1 w-[200px] max-h-[240px] overflow-y-auto">
                {move || {
                    let list = projects.get();
                    if list.is_empty() {
                        view! {
                            <Text variant=TextVariant::BodySm color=TextColor::Tertiary tag=TextTag::P class="px-3 py-2">
                                "No projects"
                            </Text>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div>
                                <button
                                    class="w-full text-left px-3 \
                                           py-1.5 text-sm \
                                           text-text-primary \
                                           hover:bg-bg-tertiary \
                                           rounded transition-colors \
                                           flex items-center gap-2"
                                    data-testid="project-picker-inbox"
                                    on:click=move |_| {
                                        set_popover_open.set(false);
                                        on_clear_project.run(task_id);
                                    }
                                >
                                    <Icon
                                        kind=IconKind::Inbox
                                        class="w-4 h-4 text-text-tertiary"
                                    />
                                    "Inbox"
                                </button>
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
                                                data-testid="project-picker-option"
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
                }}
            </div>
        </Popover>
    }
}
