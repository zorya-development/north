use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::stores::lookup_store::LookupStore;
use north_ui::{AutocompleteDropdown, SuggestionItem};

#[derive(Clone)]
struct TriggerState {
    trigger: char,
    start: usize,
    query: String,
}

fn find_trigger(value: &str, cursor: usize) -> Option<TriggerState> {
    let before = &value[..cursor];
    for trigger in ['#', '@'] {
        if let Some(pos) = before.rfind(trigger) {
            if pos == 0 || before.as_bytes()[pos - 1] == b' ' {
                let query = &before[pos + 1..];
                if query
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    return Some(TriggerState {
                        trigger,
                        start: pos,
                        query: query.to_string(),
                    });
                }
            }
        }
    }
    None
}

fn get_suggestions(lookup: &LookupStore, trigger: char, query: &str) -> Vec<SuggestionItem> {
    let query_lower = query.to_lowercase();
    match trigger {
        '#' => {
            let tags = lookup.tags.get().and_then(|r| r.ok()).unwrap_or_default();
            tags.into_iter()
                .filter(|t| query_lower.is_empty() || t.name.to_lowercase().contains(&query_lower))
                .map(|t| SuggestionItem {
                    name: t.name,
                    color: t.color,
                })
                .collect()
        }
        '@' => {
            let projects = lookup
                .projects
                .get()
                .and_then(|r| r.ok())
                .unwrap_or_default();
            projects
                .into_iter()
                .filter(|p| {
                    !p.archived
                        && (query_lower.is_empty() || p.title.to_lowercase().contains(&query_lower))
                })
                .map(|p| SuggestionItem {
                    name: p.title,
                    color: p.color,
                })
                .collect()
        }
        _ => vec![],
    }
}

fn insert_completion(value: &str, trigger_start: usize, trigger: char, name: &str) -> String {
    let before = &value[..trigger_start];
    let after_trigger = &value[trigger_start + 1..];
    let rest_start = after_trigger
        .find(|c: char| c.is_whitespace())
        .unwrap_or(after_trigger.len());
    let after = &after_trigger[rest_start..];
    format!("{before}{trigger}{name}{after} ")
}

#[component]
pub fn AutocompleteInput(
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] class: &'static str,
    #[prop(optional)] on_keydown: Option<std::sync::Arc<dyn Fn(KeyboardEvent) + Send + Sync>>,
    #[prop(optional)] autofocus: bool,
) -> impl IntoView {
    let lookup = use_context::<LookupStore>();
    let (trigger_state, set_trigger_state) = signal(None::<TriggerState>);
    let (highlighted, set_highlighted) = signal(0_usize);
    let (suggestions, set_suggestions) = signal(Vec::<SuggestionItem>::new());
    let input_ref = NodeRef::<html::Input>::new();

    if autofocus {
        Effect::new(move || {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        });
    }

    let update_suggestions = move |val: &str, cursor: usize| {
        if let Some(ref lookup) = lookup {
            if let Some(ts) = find_trigger(val, cursor) {
                let items = get_suggestions(lookup, ts.trigger, &ts.query);
                set_suggestions.set(items);
                set_trigger_state.set(Some(ts));
                set_highlighted.set(0);
                return;
            }
        }
        set_suggestions.set(vec![]);
        set_trigger_state.set(None);
    };

    let on_select = Callback::new(move |name: String| {
        if let Some(ts) = trigger_state.get_untracked() {
            let val = value.get_untracked();
            let new_val = insert_completion(&val, ts.start, ts.trigger, &name);
            set_value.set(new_val);
        }
        set_suggestions.set(vec![]);
        set_trigger_state.set(None);
    });

    view! {
        <div class="relative">
            <input
                type="text"
                node_ref=input_ref
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_value.set(val.clone());
                    if let Some(target) = ev
                        .target()
                        .and_then(|t| {
                            t.dyn_ref::<leptos::web_sys::HtmlInputElement>().cloned()
                        })
                    {
                        let cursor = target.selection_start().ok().flatten().unwrap_or(
                            val.len() as u32,
                        ) as usize;
                        update_suggestions(&val, cursor);
                    }
                }
                on:keydown=move |ev| {
                    let items = suggestions.get_untracked();
                    if !items.is_empty() {
                        match ev.key().as_str() {
                            "ArrowDown" => {
                                ev.prevent_default();
                                set_highlighted.update(|h| {
                                    *h = (*h + 1).min(items.len() - 1);
                                });
                                return;
                            }
                            "ArrowUp" => {
                                ev.prevent_default();
                                set_highlighted
                                    .update(|h| {
                                        *h = h.saturating_sub(1);
                                    });
                                return;
                            }
                            "Enter" => {
                                let idx = highlighted.get_untracked();
                                if idx < items.len() {
                                    ev.prevent_default();
                                    on_select.run(items[idx].name.clone());
                                    return;
                                }
                            }
                            "Escape" => {
                                set_suggestions.set(vec![]);
                                set_trigger_state.set(None);
                                return;
                            }
                            _ => {}
                        }
                    }
                    if let Some(ref handler) = on_keydown {
                        handler(ev);
                    }
                }
                on:blur=move |_| {
                    set_suggestions.set(vec![]);
                    set_trigger_state.set(None);
                }
                class=class
            />
            <Show when=move || !suggestions.get().is_empty()>
                <AutocompleteDropdown
                    items=suggestions.get()
                    highlighted=highlighted
                    on_select=on_select
                />
            </Show>
        </div>
    }
}

#[component]
pub fn AutocompleteTextarea(
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] class: &'static str,
    #[prop(optional, default = 3)] rows: u32,
    #[prop(optional)] on_keydown: Option<std::sync::Arc<dyn Fn(KeyboardEvent) + Send + Sync>>,
) -> impl IntoView {
    let lookup = use_context::<LookupStore>();
    let (trigger_state, set_trigger_state) = signal(None::<TriggerState>);
    let (highlighted, set_highlighted) = signal(0_usize);
    let (suggestions, set_suggestions) = signal(Vec::<SuggestionItem>::new());

    let update_suggestions = move |val: &str, cursor: usize| {
        if let Some(ref lookup) = lookup {
            if let Some(ts) = find_trigger(val, cursor) {
                let items = get_suggestions(lookup, ts.trigger, &ts.query);
                set_suggestions.set(items);
                set_trigger_state.set(Some(ts));
                set_highlighted.set(0);
                return;
            }
        }
        set_suggestions.set(vec![]);
        set_trigger_state.set(None);
    };

    let on_select = Callback::new(move |name: String| {
        if let Some(ts) = trigger_state.get_untracked() {
            let val = value.get_untracked();
            let new_val = insert_completion(&val, ts.start, ts.trigger, &name);
            set_value.set(new_val);
        }
        set_suggestions.set(vec![]);
        set_trigger_state.set(None);
    });

    view! {
        <div class="relative">
            <textarea
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_value.set(val.clone());
                    if let Some(target) = ev
                        .target()
                        .and_then(|t| {
                            t.dyn_ref::<leptos::web_sys::HtmlTextAreaElement>().cloned()
                        })
                    {
                        let cursor = target.selection_start().ok().flatten().unwrap_or(
                            val.len() as u32,
                        ) as usize;
                        update_suggestions(&val, cursor);
                    }
                }
                on:keydown=move |ev| {
                    let items = suggestions.get_untracked();
                    if !items.is_empty() {
                        match ev.key().as_str() {
                            "ArrowDown" => {
                                ev.prevent_default();
                                set_highlighted.update(|h| {
                                    *h = (*h + 1).min(items.len() - 1);
                                });
                                return;
                            }
                            "ArrowUp" => {
                                ev.prevent_default();
                                set_highlighted
                                    .update(|h| {
                                        *h = h.saturating_sub(1);
                                    });
                                return;
                            }
                            "Enter" if !ev.shift_key() => {
                                let idx = highlighted.get_untracked();
                                if idx < items.len() {
                                    ev.prevent_default();
                                    on_select.run(items[idx].name.clone());
                                    return;
                                }
                            }
                            "Escape" => {
                                set_suggestions.set(vec![]);
                                set_trigger_state.set(None);
                                return;
                            }
                            _ => {}
                        }
                    }
                    if let Some(ref handler) = on_keydown {
                        handler(ev);
                    }
                }
                on:blur=move |_| {
                    set_suggestions.set(vec![]);
                    set_trigger_state.set(None);
                }
                rows=rows
                class=class
            />
            <Show when=move || !suggestions.get().is_empty()>
                <AutocompleteDropdown
                    items=suggestions.get()
                    highlighted=highlighted
                    on_select=on_select
                />
            </Show>
        </div>
    }
}
