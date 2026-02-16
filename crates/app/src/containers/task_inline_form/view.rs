use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use north_ui::MarkdownView;

use super::controller::TaskInlineFormController;
use crate::containers::autocomplete::{AutocompleteInput, AutocompleteTextarea};

#[component]
pub fn TaskInlineFormView(ctrl: TaskInlineFormController) -> impl IntoView {
    let (title, set_title) = ctrl.title;
    let (body, set_body) = ctrl.body;
    let (preview, set_preview) = ctrl.preview;
    let is_edit = ctrl.is_edit_mode();

    let on_form_keydown = std::sync::Arc::new(move |ev: KeyboardEvent| {
        if ev.key() == "Enter" && ev.shift_key() {
            ev.prevent_default();
            ctrl.save();
        } else if ev.key() == "Escape" {
            ctrl.cancel();
        }
    });

    view! {
        <div class="border border-border rounded-xl p-3 shadow-sm \
                    focus-within:border-accent transition-colors">
            <AutocompleteInput
                value=title
                set_value=set_title
                placeholder="Task title"
                class="w-full text-sm font-semibold bg-transparent \
                       outline-none no-focus-ring \
                       placeholder:text-text-secondary \
                       text-text-primary mb-2"
                on_keydown=on_form_keydown.clone()
                autofocus=true
            />

            <Show
                when=move || preview.get()
                fallback={
                    let on_form_keydown = on_form_keydown.clone();
                    move || {
                        let on_form_keydown = on_form_keydown.clone();
                        view! {
                            <AutocompleteTextarea
                                value=body
                                set_value=set_body
                                placeholder="Description (markdown supported)"
                                rows=3
                                class="w-full text-sm bg-transparent \
                                       outline-none no-focus-ring \
                                       placeholder:text-text-tertiary \
                                       text-text-secondary resize-none"
                                on_keydown=on_form_keydown
                            />
                        }
                    }
                }
            >
                {move || {
                    let b = body.get();
                    if b.trim().is_empty() {
                        view! {
                            <p class="text-sm text-text-tertiary py-2">
                                "Nothing to preview"
                            </p>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="py-1">
                                <MarkdownView content=b/>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </Show>

            <div class="flex items-center justify-between mt-2 \
                        pt-2 border-t border-border">
                <button
                    on:click=move |_| set_preview.update(|p| *p = !*p)
                    class="text-xs text-text-tertiary \
                           hover:text-text-secondary transition-colors"
                >
                    {move || if preview.get() { "Edit" } else { "Preview" }}
                </button>
                <div class="flex gap-2">
                    <button
                        on:click=move |_| ctrl.cancel()
                        class="px-3 py-1 text-sm text-text-secondary \
                               hover:text-text-primary transition-colors"
                    >
                        "Cancel"
                    </button>
                    <button
                        on:click=move |_| ctrl.save()
                        class="px-3 py-1 text-sm bg-accent \
                               text-on-accent rounded hover:bg-accent-hover \
                               transition-colors"
                    >
                        {if is_edit { "Save" } else { "Add task" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
