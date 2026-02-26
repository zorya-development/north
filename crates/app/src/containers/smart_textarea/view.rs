use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use north_dto::{Project, Tag};
use north_ui::{AutocompleteDropdown, SuggestionItem};

use crate::components::mirror_overlay::MirrorOverlay;
use crate::containers::autocomplete::{find_trigger, get_suggestions, insert_completion};

/// Pure view for SmartTextarea — receives tags/projects as signals,
/// no direct store access.
#[component]
pub fn SmartTextareaView(
    value: RwSignal<String>,
    placeholder: &'static str,
    class: &'static str,
    node_ref: Option<NodeRef<leptos::html::Textarea>>,
    data_testid: &'static str,

    // Feature toggles
    autocomplete: bool,
    mirror_overlay: bool,
    auto_resize: bool,
    multiline: bool,
    strip_newlines: bool,

    // Behavior callbacks
    on_submit: Option<Callback<()>>,
    on_close: Option<Callback<()>>,
    on_blur: Option<Callback<()>>,
    on_input: Option<Callback<()>>,
    autofocus: bool,
    rows: u32,

    // Data from controller
    tags: Signal<Vec<Tag>>,
    projects: Signal<Vec<Project>>,
) -> impl IntoView {
    let textarea_ref = node_ref.unwrap_or_default();

    // Autocomplete state
    let (trigger_state, set_trigger_state) =
        signal(None::<crate::containers::autocomplete::TriggerState>);
    let (highlighted, set_highlighted) = signal(0_usize);
    let (suggestions, set_suggestions) = signal(Vec::<SuggestionItem>::new());

    // Auto-resize helper
    let do_auto_resize = move || {
        if !auto_resize {
            return;
        }
        if let Some(el) = textarea_ref.get_untracked() {
            if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                let _ = html_el.style().set_property("height", "auto");
                let scroll_h = html_el.scroll_height();
                let _ = html_el
                    .style()
                    .set_property("height", &format!("{scroll_h}px"));
            }
        }
    };

    // Autofocus + initial auto-resize
    if autofocus {
        Effect::new(move || {
            if let Some(el) = textarea_ref.get() {
                let _ = el.focus();
                do_auto_resize();
            }
        });
    }

    // Auto-resize on value change (for externally set values like sync_drafts)
    if auto_resize {
        Effect::new(move || {
            let _ = value.get();
            if textarea_ref.get().is_some() {
                do_auto_resize();
            }
        });
    }

    let update_suggestions = move |val: &str, cursor: usize| {
        if !autocomplete {
            return;
        }
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
            value.set(new_val);
        }
        set_suggestions.set(vec![]);
        set_trigger_state.set(None);
    });

    // Build combined class: add textarea-mirror when overlay enabled
    let combined_class = if mirror_overlay {
        if class.is_empty() {
            "textarea-mirror"
        } else {
            // Leak is fine for static-lifetime class strings built once per component
            Box::leak(format!("{class} textarea-mirror").into_boxed_str()) as &'static str
        }
    } else {
        class
    };

    view! {
        <div class="relative">
            {mirror_overlay.then(|| {
                view! { <MirrorOverlay value=Signal::derive(move || value.get()) /> }
            })}
            <textarea
                data-testid=data_testid
                node_ref=textarea_ref
                placeholder=placeholder
                rows=rows
                class=combined_class
                prop:value=move || value.get()
                on:input=move |ev| {
                    let val = event_target_value(&ev);

                    // Strip newlines if enabled
                    let val = if strip_newlines {
                        let cleaned = val.replace('\n', " ").replace('\r', "");
                        if cleaned != val {
                            value.set(cleaned.clone());
                            cleaned
                        } else {
                            value.set(val.clone());
                            val
                        }
                    } else {
                        value.set(val.clone());
                        val
                    };

                    do_auto_resize();

                    // Update autocomplete suggestions
                    if autocomplete {
                        if let Some(target) = ev
                            .target()
                            .and_then(|t| {
                                t.dyn_ref::<web_sys::HtmlTextAreaElement>().cloned()
                            })
                        {
                            let cursor = target.selection_start().ok().flatten().unwrap_or(
                                val.len() as u32,
                            ) as usize;
                            update_suggestions(&val, cursor);
                        }
                    }

                    if let Some(cb) = on_input {
                        cb.run(());
                    }
                }
                on:keydown=move |ev| {
                    // Priority 1-4: autocomplete handling
                    if autocomplete {
                        let items = suggestions.get_untracked();
                        if !items.is_empty() {
                            match ev.key().as_str() {
                                "ArrowDown" => {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    set_highlighted.update(|h| {
                                        *h = (*h + 1).min(items.len() - 1);
                                    });
                                    return;
                                }
                                "ArrowUp" => {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    set_highlighted.update(|h| {
                                        *h = h.saturating_sub(1);
                                    });
                                    return;
                                }
                                "Enter" => {
                                    let idx = highlighted.get_untracked();
                                    if idx < items.len() {
                                        ev.prevent_default();
                                        ev.stop_propagation();
                                        on_select.run(items[idx].name.clone());
                                        return;
                                    }
                                }
                                "Escape" => {
                                    // Close dropdown only — don't fire on_close
                                    ev.stop_propagation();
                                    set_suggestions.set(vec![]);
                                    set_trigger_state.set(None);
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }

                    // Priority 5: Ctrl/Cmd+Enter inserts newline
                    if multiline && ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) {
                        ev.prevent_default();
                        ev.stop_propagation();
                        if let Some(el) = textarea_ref.get_untracked() {
                            let ta: &web_sys::HtmlTextAreaElement = &el;
                            crate::libs::insert_newline_at_cursor(ta);
                        }
                        return;
                    }

                    // Priority 6: plain Enter → submit
                    if ev.key() == "Enter" {
                        if let Some(cb) = on_submit {
                            ev.prevent_default();
                            ev.stop_propagation();
                            cb.run(());
                            return;
                        }
                    }

                    // Priority 7: Escape → close (if callback provided)
                    // Otherwise let event bubble (e.g. to modal's window listener)
                    if ev.key() == "Escape" {
                        if let Some(cb) = on_close {
                            ev.prevent_default();
                            ev.stop_propagation();
                            cb.run(());
                        }
                    }
                }
                on:blur=move |_| {
                    // Clear autocomplete suggestions
                    if autocomplete {
                        set_suggestions.set(vec![]);
                        set_trigger_state.set(None);
                    }
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            {(autocomplete).then(|| {
                view! {
                    <Show when=move || !suggestions.get().is_empty()>
                        <AutocompleteDropdown
                            items=suggestions.get()
                            highlighted=highlighted
                            on_select=on_select
                        />
                    </Show>
                }
            })}
        </div>
    }
}
