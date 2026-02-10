use leptos::prelude::*;
use north_domain::{Tag, TagInfo};
use north_ui::{Icon, IconKind, Popover};

#[component]
pub fn TagPickerView(
    task_id: i64,
    tags: Vec<TagInfo>,
    popover_open: ReadSignal<bool>,
    set_popover_open: WriteSignal<bool>,
    all_tags: Resource<Result<Vec<Tag>, ServerFnError>>,
    current_tags: ReadSignal<Vec<String>>,
    set_current_tags: WriteSignal<Vec<String>>,
    on_set_tags: Callback<(i64, Vec<String>)>,
) -> impl IntoView {
    let (new_tag_input, set_new_tag_input) = signal(String::new());
    let has_tags = !tags.is_empty();

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
                names.push(name);
                set_current_tags.set(names.clone());
                on_set_tags.run((task_id, names));
            }
            set_new_tag_input.set(String::new());
        }
    };

    view! {
        <Popover
            open=popover_open
            set_open=set_popover_open
            trigger=Box::new({
                let tags = tags.clone();
                move || {
                    if has_tags {
                        let tags_clone = tags.clone();
                        view! {
                            <div class="flex items-center gap-1 flex-wrap">
                                {tags_clone
                                    .into_iter()
                                    .map(|tag| {
                                        let name = tag.name.clone();
                                        let color = tag.color.clone();
                                        let remove_name = name.clone();
                                        view! {
                                            <span
                                                class="text-xs px-2 py-0.5 rounded-full \
                                                       inline-flex items-center gap-1"
                                                style=format!(
                                                    "background-color: {}22; color: {}; \
                                                     border: 1px solid {}44",
                                                    color, color, color,
                                                )
                                            >
                                                {name}
                                                <span
                                                    class="cursor-pointer hover:opacity-70"
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
                                                </span>
                                            </span>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                                <button
                                    class="text-text-tertiary hover:text-text-secondary \
                                           hover:bg-bg-tertiary px-1 py-0.5 rounded \
                                           transition-colors text-xs"
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
                                <Icon kind=IconKind::Tag class="w-3 h-3"/>
                                "Tags"
                            </button>
                        }
                        .into_any()
                    }
                }
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
                <Suspense fallback=move || {
                    view! {
                        <div class="px-3 py-2 text-xs text-text-tertiary">
                            "Loading..."
                        </div>
                    }
                }>
                    {move || {
                        Suspend::new(async move {
                            match all_tags.await {
                                Ok(list) => {
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
