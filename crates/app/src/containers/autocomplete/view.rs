use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use north_dto::{Project, ProjectStatus, Tag};
use north_ui::{AutocompleteDropdown, SuggestionItem};

#[derive(Clone)]
pub struct TriggerState {
    pub trigger: char,
    pub start: usize,
    pub query: String,
}

pub fn find_trigger(value: &str, cursor: usize) -> Option<TriggerState> {
    if cursor > value.len() || !value.is_char_boundary(cursor) {
        return None;
    }
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

pub fn get_suggestions(
    tags: &[Tag],
    projects: &[Project],
    trigger: char,
    query: &str,
) -> Vec<SuggestionItem> {
    let query_lower = query.to_lowercase();
    match trigger {
        '#' => tags
            .iter()
            .filter(|t| query_lower.is_empty() || t.name.to_lowercase().contains(&query_lower))
            .map(|t| SuggestionItem {
                name: t.name.clone(),
                color: t.color.clone(),
            })
            .collect(),
        '@' => projects
            .iter()
            .filter(|p| {
                p.status == ProjectStatus::Active
                    && (query_lower.is_empty() || p.title.to_lowercase().contains(&query_lower))
            })
            .map(|p| SuggestionItem {
                name: p.title.clone(),
                color: p.color.clone(),
            })
            .collect(),
        _ => vec![],
    }
}

pub fn insert_completion(value: &str, trigger_start: usize, trigger: char, name: &str) -> String {
    let before = &value[..trigger_start];
    let after_trigger = &value[trigger_start + 1..];
    let rest_start = after_trigger
        .find(|c: char| c.is_whitespace())
        .unwrap_or(after_trigger.len());
    let after = &after_trigger[rest_start..];
    format!("{before}{trigger}{name}{after} ")
}

#[component]
pub fn AutocompleteInputView(
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] class: &'static str,
    on_keydown: Option<std::sync::Arc<dyn Fn(KeyboardEvent) + Send + Sync>>,
    autofocus: bool,
    on_blur: Option<Callback<()>>,
    node_ref: Option<NodeRef<html::Input>>,
    tags: Signal<Vec<Tag>>,
    projects: Signal<Vec<Project>>,
) -> impl IntoView {
    let (trigger_state, set_trigger_state) = signal(None::<TriggerState>);
    let (highlighted, set_highlighted) = signal(0_usize);
    let (suggestions, set_suggestions) = signal(Vec::<SuggestionItem>::new());
    let input_ref = node_ref.unwrap_or_default();

    if autofocus {
        Effect::new(move || {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        });
    }

    let update_suggestions = move |val: &str, cursor: usize| {
        if let Some(ts) = find_trigger(val, cursor) {
            let items = get_suggestions(&tags.get(), &projects.get(), ts.trigger, &ts.query);
            set_suggestions.set(items);
            set_trigger_state.set(Some(ts));
            set_highlighted.set(0);
            return;
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
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
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
