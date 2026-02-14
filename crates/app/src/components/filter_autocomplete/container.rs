use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use north_domain::{detect_completion_context, DslCompletionContext, FilterField};
use north_ui::{AutocompleteDropdown, SuggestionItem};

use crate::stores::lookup_store::LookupStore;
use north_stores::AppStore;

const FIELD_NAMES: &[&str] = &[
    "title", "body", "project", "tags", "status", "due_date", "start_at", "created", "updated",
];

const STATUS_VALUES: &[&str] = &["ACTIVE", "OPEN", "COMPLETED", "DONE"];

const KEYWORDS: &[&str] = &["AND", "OR", "NOT", "ORDER BY"];

fn get_dsl_suggestions(
    lookup: &LookupStore,
    app_store: &AppStore,
    ctx: &DslCompletionContext,
) -> Vec<SuggestionItem> {
    match ctx {
        DslCompletionContext::FieldName { partial, .. } => {
            let partial_lower = partial.to_lowercase();
            FIELD_NAMES
                .iter()
                .filter(|f| partial_lower.is_empty() || f.starts_with(&partial_lower))
                .map(|f| SuggestionItem {
                    name: f.to_string(),
                    color: "#6b7280".into(),
                })
                .collect()
        }
        DslCompletionContext::FieldValue { field, partial, .. }
        | DslCompletionContext::ArrayValue { field, partial, .. } => {
            let partial_lower = partial.to_lowercase();
            match field {
                FilterField::Tags => {
                    let tags = lookup.tags.get().and_then(|r| r.ok()).unwrap_or_default();
                    tags.into_iter()
                        .filter(|t| {
                            partial_lower.is_empty()
                                || t.name.to_lowercase().contains(&partial_lower)
                        })
                        .map(|t| SuggestionItem {
                            name: t.name,
                            color: t.color,
                        })
                        .collect()
                }
                FilterField::Project => {
                    let projects = app_store.projects.get();
                    projects
                        .into_iter()
                        .filter(|p| {
                            p.status == north_domain::ProjectStatus::Active
                                && (partial_lower.is_empty()
                                    || p.title.to_lowercase().contains(&partial_lower))
                        })
                        .map(|p| SuggestionItem {
                            name: p.title,
                            color: p.color,
                        })
                        .collect()
                }
                FilterField::Status => STATUS_VALUES
                    .iter()
                    .filter(|s| {
                        partial_lower.is_empty() || s.to_lowercase().starts_with(&partial_lower)
                    })
                    .map(|s| SuggestionItem {
                        name: s.to_string(),
                        color: "#6b7280".into(),
                    })
                    .collect(),
                _ => vec![],
            }
        }
        DslCompletionContext::Keyword { partial, .. } => {
            let partial_upper = partial.to_uppercase();
            KEYWORDS
                .iter()
                .filter(|k| partial_upper.is_empty() || k.starts_with(&partial_upper))
                .map(|k| SuggestionItem {
                    name: k.to_string(),
                    color: "#6b7280".into(),
                })
                .collect()
        }
        DslCompletionContext::None => vec![],
    }
}

fn insert_dsl_completion(
    value: &str,
    start: usize,
    cursor: usize,
    name: &str,
    ctx: &DslCompletionContext,
) -> (String, usize) {
    let before = &value[..start];
    let after = &value[cursor..];

    let needs_quotes = name.contains(' ')
        || matches!(
            ctx,
            DslCompletionContext::FieldValue {
                field: FilterField::Tags | FilterField::Project,
                ..
            } | DslCompletionContext::ArrayValue {
                field: FilterField::Tags | FilterField::Project,
                ..
            }
        );

    let insertion = if needs_quotes {
        format!("'{name}' ")
    } else {
        format!("{name} ")
    };

    let new_cursor = before.len() + insertion.len();
    let new_value = format!("{before}{insertion}{after}");
    (new_value, new_cursor)
}

#[component]
pub fn FilterAutocompleteTextarea(
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] class: &'static str,
    #[prop(optional, default = 3)] rows: u32,
    #[prop(optional)] on_submit: Option<Callback<()>>,
) -> impl IntoView {
    let lookup = use_context::<LookupStore>();
    let app_store = use_context::<AppStore>();
    let (highlighted, set_highlighted) = signal(0_usize);
    let (suggestions, set_suggestions) = signal(Vec::<SuggestionItem>::new());
    let (completion_ctx, set_completion_ctx) = signal(DslCompletionContext::None);
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    let update_suggestions = move |val: &str, cursor: usize| {
        if let (Some(ref lookup), Some(ref app_store)) = (&lookup, &app_store) {
            let ctx = detect_completion_context(val, cursor);
            let items = get_dsl_suggestions(lookup, app_store, &ctx);
            set_suggestions.set(items);
            set_completion_ctx.set(ctx);
            set_highlighted.set(0);
        }
    };

    let on_select = Callback::new(move |name: String| {
        let val = value.get_untracked();
        let ctx = completion_ctx.get_untracked();

        let (start, cursor) = match &ctx {
            DslCompletionContext::FieldName { start, partial, .. } => {
                (*start, start + partial.len())
            }
            DslCompletionContext::FieldValue { start, partial, .. } => {
                (*start, start + partial.len())
            }
            DslCompletionContext::ArrayValue { start, partial, .. } => {
                (*start, start + partial.len())
            }
            DslCompletionContext::Keyword { start, partial, .. } => (*start, start + partial.len()),
            DslCompletionContext::None => return,
        };

        let (new_val, new_cursor) = insert_dsl_completion(&val, start, cursor, &name, &ctx);
        set_value.set(new_val);
        set_suggestions.set(vec![]);
        set_completion_ctx.set(DslCompletionContext::None);

        // Set cursor position after insertion
        if let Some(el) = textarea_ref.get() {
            let el: &leptos::web_sys::HtmlTextAreaElement = &el;
            let _ = el.set_selection_start(Some(new_cursor as u32));
            let _ = el.set_selection_end(Some(new_cursor as u32));
            let _ = el.focus();
        }
    });

    view! {
        <div class="relative flex flex-col">
            <textarea
                node_ref=textarea_ref
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
                        let cursor = target
                            .selection_start()
                            .ok()
                            .flatten()
                            .unwrap_or(val.len() as u32) as usize;
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
                                set_highlighted.update(|h| {
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
                            "Tab" => {
                                let idx = highlighted.get_untracked();
                                if idx < items.len() {
                                    ev.prevent_default();
                                    on_select.run(items[idx].name.clone());
                                    return;
                                }
                            }
                            "Escape" => {
                                set_suggestions.set(vec![]);
                                set_completion_ctx.set(DslCompletionContext::None);
                                return;
                            }
                            _ => {}
                        }
                    } else if ev.key() == "Enter" && !ev.shift_key() {
                        if let Some(cb) = on_submit {
                            ev.prevent_default();
                            cb.run(());
                        }
                    }
                }
                on:blur=move |_| {
                    set_suggestions.set(vec![]);
                    set_completion_ctx.set(DslCompletionContext::None);
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
