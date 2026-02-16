use leptos::prelude::*;
use north_dto::{Tag, TagInfo};
use north_ui::{Icon, IconKind, Popover};

#[component]
pub fn TagPickerView(
    task_id: i64,
    display_tags: ReadSignal<Vec<TagInfo>>,
    set_display_tags: WriteSignal<Vec<TagInfo>>,
    popover_open: ReadSignal<bool>,
    set_popover_open: WriteSignal<bool>,
    all_tags: Memo<Vec<Tag>>,
    current_tags: ReadSignal<Vec<String>>,
    set_current_tags: WriteSignal<Vec<String>>,
    on_set_tags: Callback<(i64, Vec<String>)>,
    #[prop(default = false)] icon_only: bool,
    #[prop(default = false)] always_visible: bool,
) -> impl IntoView {
    let (new_tag_input, set_new_tag_input) = signal(String::new());

    let toggle_tag = move |name: String| {
        let mut names = current_tags.get_untracked();
        if let Some(pos) = names.iter().position(|n| *n == name) {
            names.remove(pos);
        } else {
            names.push(name);
        }
        set_current_tags.set(names.clone());
        on_set_tags.run((task_id, names));
    };

    let add_new_tag = move || {
        let name = new_tag_input.get_untracked().trim().to_string();
        if !name.is_empty() {
            let mut names = current_tags.get_untracked();
            if !names.contains(&name) {
                names.push(name.clone());
                set_current_tags.set(names.clone());

                let mut tags = display_tags.get_untracked();
                tags.push(TagInfo {
                    name,
                    color: "#6b7280".to_string(),
                });
                set_display_tags.set(tags);

                on_set_tags.run((task_id, names));
            }
            set_new_tag_input.set(String::new());
        }
    };

    view! {
        <Popover
            open=popover_open
            set_open=set_popover_open
            trigger=Box::new(move || {
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
                            aria-label="Set tags"
                        >
                            <Icon
                                kind=IconKind::Tag
                                class="w-4 h-4"
                            />
                        </button>
                    }
                    .into_any();
                }
                view! {
                    {move || {
                        let tags = display_tags.get();
                        let names = current_tags.get();
                        let active_tags: Vec<&TagInfo> = tags
                            .iter()
                            .filter(|t| names.contains(&t.name))
                            .collect();
                        if !active_tags.is_empty() {
                            view! {
                                <div class="flex items-center gap-0.5 flex-wrap">
                                    {active_tags
                                        .into_iter()
                                        .map(|tag| {
                                            let name = tag.name.clone();
                                            let color = tag.color.clone();
                                            let remove_name = name.clone();
                                            view! {
                                                <span
                                                    class="group/tag text-xs \
                                                           inline-flex items-center \
                                                           gap-0.5 select-none"
                                                    style=format!(
                                                        "color: {}",
                                                        color,
                                                    )
                                                >
                                                    <Icon
                                                        kind=IconKind::Tag
                                                        class="w-3 h-3"
                                                    />
                                                    {name}
                                                    <button
                                                        class=if always_visible {
                                                            "cursor-pointer \
                                                             hover:opacity-70"
                                                        } else {
                                                            "cursor-pointer \
                                                             hover:opacity-70 \
                                                             hidden \
                                                             group-hover/tag:inline"
                                                        }
                                                        on:click=move |ev| {
                                                            ev.stop_propagation();
                                                            let mut names =
                                                                current_tags.get_untracked();
                                                            names.retain(|n| {
                                                                *n != remove_name
                                                            });
                                                            set_current_tags
                                                                .set(names.clone());
                                                            on_set_tags
                                                                .run((task_id, names));
                                                        }
                                                    >
                                                        "\u{00d7}"
                                                    </button>
                                                </span>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                    <button
                                        class="text-text-tertiary \
                                               hover:text-text-secondary \
                                               hover:bg-bg-tertiary px-1 py-0.5 \
                                               rounded transition-colors text-xs \
                                               cursor-pointer"
                                        on:click=move |_| {
                                            set_popover_open.update(|o| *o = !*o);
                                        }
                                    >
                                        "+"
                                    </button>
                                </div>
                            }
                            .into_any()
                        } else {
                            {
                                let vis_class = if always_visible {
                                    "inline-flex items-center gap-1 \
                                     text-text-tertiary \
                                     hover:text-text-secondary \
                                     hover:bg-bg-tertiary px-1.5 py-0.5 \
                                     rounded transition-colors \
                                     cursor-pointer select-none"
                                } else {
                                    "items-center gap-1 \
                                     text-text-tertiary \
                                     hover:text-text-secondary \
                                     hover:bg-bg-tertiary px-1.5 py-0.5 \
                                     rounded transition-colors \
                                     cursor-pointer select-none \
                                     hidden group-hover:inline-flex"
                                };
                                view! {
                                    <button
                                        class=vis_class
                                        on:click=move |_| {
                                            set_popover_open
                                                .update(|o| *o = !*o);
                                        }
                                    >
                                        <Icon
                                            kind=IconKind::Tag
                                            class="w-3 h-3"
                                        />
                                        "None"
                                    </button>
                                }
                                .into_any()
                            }
                        }
                    }}
                }
                .into_any()
            })
        >
            <div class="p-1 w-[200px] max-h-[280px] overflow-y-auto">
                <div class="px-2 py-1">
                    <input
                        type="text"
                        class="w-full bg-bg-input border border-border \
                               rounded px-2 py-1 text-xs \
                               text-text-primary \
                               placeholder:text-text-tertiary \
                               focus:outline-none focus:border-accent"
                        placeholder="New tag..."
                        prop:value=move || new_tag_input.get()
                        on:input=move |ev| {
                            set_new_tag_input
                                .set(event_target_value(&ev));
                        }
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                ev.prevent_default();
                                add_new_tag();
                            }
                        }
                    />
                </div>
                {move || {
                    let list = all_tags.get();
                    if list.is_empty() {
                        view! {
                            <div class="px-3 py-2 text-xs \
                                        text-text-tertiary">
                                "No tags yet"
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div>
                                {list
                                    .into_iter()
                                    .map(|tag| {
                                        let name =
                                            tag.name.clone();
                                        let color =
                                            tag.color.clone();
                                        let toggle_name =
                                            name.clone();
                                        let check_name =
                                            name.clone();
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
                                                    toggle_tag(
                                                        toggle_name.clone(),
                                                    );
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
                                                <span class="flex-1">
                                                    {name}
                                                </span>
                                                {move || {
                                                    let names =
                                                        current_tags.get();
                                                    if names.contains(
                                                        &check_name,
                                                    ) {
                                                        Some(view! {
                                                            <Icon
                                                                kind=IconKind::Check
                                                                class="w-3 h-3 \
                                                                       text-accent"
                                                            />
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                }}
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
