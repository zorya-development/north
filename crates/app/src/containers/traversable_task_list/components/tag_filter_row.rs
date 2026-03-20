use std::collections::BTreeMap;

use leptos::prelude::*;
use north_dto::tag::parse_kv;
use north_ui::Popover;

/// A single k:v tag value entry: `(full_name, value, color)`.
type KvValue = (String, String, String);

/// Grouped representation of available tags for the filter bar.
#[derive(Clone, PartialEq)]
struct GroupedTags {
    /// Simple tags: `(name, color)`
    simple: Vec<(String, String)>,
    /// K:V groups: `(key, values)`
    kv: Vec<(String, Vec<KvValue>)>,
}

fn group_available_tags(tags: &[(String, String)]) -> GroupedTags {
    let mut simple = Vec::new();
    let mut kv: BTreeMap<String, Vec<KvValue>> = BTreeMap::new();
    for (name, color) in tags {
        if let Some((key, value)) = parse_kv(name) {
            kv.entry(key.to_string()).or_default().push((
                name.clone(),
                value.to_string(),
                color.clone(),
            ));
        } else {
            simple.push((name.clone(), color.clone()));
        }
    }
    GroupedTags {
        simple,
        kv: kv.into_iter().collect(),
    }
}

#[component]
pub fn TagFilterRow(
    available_tags: Memo<Vec<(String, String)>>,
    active_tag_names: RwSignal<Vec<String>>,
    on_toggle: Callback<String>,
) -> impl IntoView {
    let has_tags = Memo::new(move |_| !available_tags.get().is_empty());

    let grouped = Memo::new(move |_| group_available_tags(&available_tags.get()));

    view! {
        <Show when=move || has_tags.get()>
            <div
                data-testid="tag-filter-bar"
                class="flex flex-wrap items-center gap-2"
            >
                // Simple tags: flat toggle buttons (unchanged)
                <For
                    each=move || grouped.get().simple
                    key=|(name, _)| name.clone()
                    children=move |(name, _color)| {
                        let name_for_click = name.clone();
                        let name_for_active = name.clone();
                        let display = format!("#{name}");
                        let on_toggle = on_toggle;

                        let is_active = Memo::new(move |_| {
                            active_tag_names.get().contains(&name_for_active)
                        });

                        view! {
                            <button
                                data-testid="tag-filter-button"
                                data-active=move || is_active.get().to_string()
                                class=move || {
                                    if is_active.get() {
                                        "text-xs text-accent cursor-pointer \
                                         transition-colors"
                                    } else {
                                        "text-xs text-text-tertiary \
                                         hover:text-text-secondary \
                                         cursor-pointer transition-colors"
                                    }
                                }
                                on:click=move |_| {
                                    on_toggle.run(name_for_click.clone());
                                }
                            >
                                {display.clone()}
                            </button>
                        }
                    }
                />
                // K:V groups: grouped chips with popover
                <For
                    each=move || grouped.get().kv
                    key=|(key, _)| key.clone()
                    children=move |(key, _values)| {
                        let (popover_open, set_popover_open) = signal(false);
                        let on_toggle = on_toggle;

                        let key_for_label = key.clone();
                        let key_for_values = key.clone();
                        // Derive values reactively from the grouped Memo so new
                        // values under the same key appear without re-keying.
                        let current_values = Memo::new(move |_| {
                            grouped
                                .get()
                                .kv
                                .iter()
                                .find(|(k, _)| k == &key_for_values)
                                .map(|(_, v)| v.clone())
                                .unwrap_or_default()
                        });

                        let active_values = Memo::new(move |_| {
                            let active = active_tag_names.get();
                            current_values
                                .get()
                                .iter()
                                .filter(|(full_name, _, _)| active.contains(full_name))
                                .map(|(_, val, _)| val.clone())
                                .collect::<Vec<_>>()
                        });

                        let has_active = Memo::new(move |_| !active_values.get().is_empty());

                        let chip_label = Memo::new(move |_| {
                            let selected = active_values.get();
                            if selected.is_empty() {
                                format!("#{}:all", key_for_label)
                            } else {
                                format!("#{}:{}", key_for_label, selected.join(","))
                            }
                        });

                        view! {
                            <Popover
                                open=popover_open
                                set_open=set_popover_open
                                trigger=Box::new(move || {
                                    view! {
                                        <button
                                            data-testid="kv-filter-chip"
                                            class=move || {
                                                if has_active.get() {
                                                    "text-xs text-accent cursor-pointer \
                                                     transition-colors"
                                                } else {
                                                    "text-xs text-text-tertiary \
                                                     hover:text-text-secondary \
                                                     cursor-pointer transition-colors"
                                                }
                                            }
                                            on:click=move |_| {
                                                set_popover_open.set(!popover_open.get());
                                            }
                                        >
                                            {move || chip_label.get()}
                                        </button>
                                    }.into_any()
                                })
                            >
                                <div data-testid="kv-filter-popover" class="p-2 min-w-[140px]">
                                    <For
                                        each=move || current_values.get()
                                        key=|(full_name, _, _)| full_name.clone()
                                        children=move |(full_name, value, _color)| {
                                            let full_name_for_check = full_name.clone();
                                            let full_name_for_click = full_name.clone();
                                            let on_toggle = on_toggle;

                                            let is_checked = Memo::new(move |_| {
                                                active_tag_names.get().contains(&full_name_for_check)
                                            });

                                            view! {
                                                <label
                                                    data-testid="kv-filter-option"
                                                    class="flex items-center gap-2 px-1 py-0.5 \
                                                           text-xs text-text-secondary \
                                                           hover:text-text-primary cursor-pointer"
                                                >
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || is_checked.get()
                                                        class="accent-accent"
                                                        on:change=move |_| {
                                                            on_toggle.run(full_name_for_click.clone());
                                                        }
                                                    />
                                                    {value.clone()}
                                                </label>
                                            }
                                        }
                                    />
                                </div>
                            </Popover>
                        }
                    }
                />
            </div>
        </Show>
    }
}
