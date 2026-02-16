use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use north_ui::{AutocompleteDropdown, SuggestionItem};

use north_stores::AppStore;

#[component]
pub fn FilterAutocompleteTextarea(
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] class: &'static str,
    #[prop(optional, default = 3)] rows: u32,
    #[prop(optional)] on_submit: Option<Callback<()>>,
) -> impl IntoView {
    let app_store = expect_context::<AppStore>();
    let filter_dsl = app_store.filter_dsl;

    let (highlighted, set_highlighted) = signal(0_usize);
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    let suggestions_view = Memo::new(move |_| {
        filter_dsl
            .suggestions()
            .get()
            .into_iter()
            .map(|s| SuggestionItem {
                name: s.label,
                color: if s.color.is_empty() {
                    "#6b7280".into()
                } else {
                    s.color
                },
            })
            .collect::<Vec<_>>()
    });

    let on_select = Callback::new(move |name: String| {
        let items = filter_dsl.suggestions().get_untracked();
        if let Some(suggestion) = items.iter().find(|s| s.label == name) {
            let cursor = get_cursor(&textarea_ref);
            let (_, new_cursor) = filter_dsl.apply_completion(suggestion, cursor);

            // Set cursor position after insertion
            if let Some(el) = textarea_ref.get() {
                let el: &leptos::web_sys::HtmlTextAreaElement = &el;
                let _ = el.set_selection_start(Some(new_cursor as u32));
                let _ = el.set_selection_end(Some(new_cursor as u32));
                let _ = el.focus();
            }
        }
    });

    view! {
        <div class="relative flex flex-col">
            <textarea
                node_ref=textarea_ref
                placeholder=placeholder
                prop:value=move || filter_dsl.query().get()
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    filter_dsl.set_query(val.clone());
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
                        filter_dsl.update_completions(cursor);
                    }
                }
                on:keydown=move |ev| {
                    let items = suggestions_view.get_untracked();
                    if !items.is_empty() {
                        match ev.key().as_str() {
                            "ArrowDown" => {
                                ev.prevent_default();
                                set_highlighted.update(|h| {
                                    *h = (*h + 1).min(items.len() - 1);
                                });
                            }
                            "ArrowUp" => {
                                ev.prevent_default();
                                set_highlighted.update(|h| {
                                    *h = h.saturating_sub(1);
                                });
                            }
                            "Enter" if !ev.shift_key() => {
                                let idx = highlighted.get_untracked();
                                if idx < items.len() {
                                    ev.prevent_default();
                                    on_select.run(items[idx].name.clone());
                                }
                            }
                            "Tab" => {
                                let idx = highlighted.get_untracked();
                                if idx < items.len() {
                                    ev.prevent_default();
                                    on_select.run(items[idx].name.clone());
                                }
                            }
                            "Escape" => {
                                filter_dsl.clear_suggestions();
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
                    filter_dsl.clear_suggestions();
                }
                rows=rows
                class=class
            />
            <Show when=move || !suggestions_view.get().is_empty()>
                <AutocompleteDropdown
                    items=suggestions_view.get()
                    highlighted=highlighted
                    on_select=on_select
                />
            </Show>
        </div>
    }
}

fn get_cursor(textarea_ref: &NodeRef<leptos::html::Textarea>) -> usize {
    textarea_ref
        .get()
        .and_then(|el| {
            let el: &leptos::web_sys::HtmlTextAreaElement = &el;
            el.selection_start().ok().flatten()
        })
        .unwrap_or(0) as usize
}
