use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

use north_ui::MarkdownView;

use crate::components::autocomplete::{AutocompleteInput, AutocompleteTextarea};

#[component]
pub fn InlineTaskForm<F>(on_submit: F) -> impl IntoView
where
    F: Fn(String, Option<String>) + Send + Sync + 'static,
{
    let (expanded, set_expanded) = signal(false);
    let (title, set_title) = signal(String::new());
    let (body, set_body) = signal(String::new());
    let (preview, set_preview) = signal(false);
    let on_submit = std::sync::Arc::new(on_submit);

    let on_cancel = std::sync::Arc::new(move |_: leptos::ev::MouseEvent| {
        set_expanded.set(false);
        set_title.set(String::new());
        set_body.set(String::new());
        set_preview.set(false);
    });

    let on_save = {
        let on_submit = on_submit.clone();
        std::sync::Arc::new(move |_: leptos::ev::MouseEvent| {
            let t = title.get_untracked().trim().to_string();
            if !t.is_empty() {
                let b = body.get_untracked().trim().to_string();
                let body_opt = if b.is_empty() { None } else { Some(b) };
                on_submit(t, body_opt);
                set_expanded.set(false);
                set_title.set(String::new());
                set_body.set(String::new());
                set_preview.set(false);
            }
        })
    };

    let on_title_keydown = {
        let on_submit = on_submit.clone();
        std::sync::Arc::new(move |ev: KeyboardEvent| {
            if ev.key() == "Enter" && ev.shift_key() {
                ev.prevent_default();
                let t = title.get_untracked().trim().to_string();
                if !t.is_empty() {
                    let b = body.get_untracked().trim().to_string();
                    let body_opt = if b.is_empty() { None } else { Some(b) };
                    on_submit(t, body_opt);
                    set_expanded.set(false);
                    set_title.set(String::new());
                    set_body.set(String::new());
                    set_preview.set(false);
                }
            } else if ev.key() == "Escape" {
                set_expanded.set(false);
                set_title.set(String::new());
                set_body.set(String::new());
                set_preview.set(false);
            }
        })
    };

    view! {
        <Show
            when=move || expanded.get()
            fallback=move || {
                view! {
                    <button
                        on:click=move |_| set_expanded.set(true)
                        class="flex items-center gap-2 p-4 w-full text-left \
                               border border-border rounded-xl \
                               hover:border-accent transition-colors"
                    >
                        <span class="text-accent text-sm font-medium">"+"</span>
                        <span class="text-sm text-text-secondary">
                            "Add a task..."
                        </span>
                    </button>
                }
            }
        >
            {
                let on_title_keydown = on_title_keydown.clone();
                let on_cancel = on_cancel.clone();
                let on_save = on_save.clone();
                move || {
                    let on_title_keydown = on_title_keydown.clone();
                    let on_cancel = on_cancel.clone();
                    let on_save = on_save.clone();
                    view! {
                        <div class="border border-border rounded-xl p-3 shadow-sm \
                                    focus-within:border-accent transition-colors">
                            <AutocompleteInput
                                value=title
                                set_value=set_title
                                placeholder="Task title"
                                class="w-full text-sm font-semibold bg-transparent \
                                       outline-none placeholder:text-text-secondary \
                                       text-text-primary mb-2"
                                on_keydown=on_title_keydown.clone()
                            />

                            <Show
                                when=move || preview.get()
                                fallback=move || {
                                    view! {
                                        <AutocompleteTextarea
                                            value=body
                                            set_value=set_body
                                            placeholder="Description (markdown supported)"
                                            rows=3
                                            class="w-full text-sm bg-transparent \
                                                   outline-none \
                                                   placeholder:text-text-tertiary \
                                                   text-text-secondary resize-none"
                                        />
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
                                    {move || {
                                        if preview.get() { "Edit" } else { "Preview" }
                                    }}
                                </button>
                                <div class="flex gap-2">
                                    <button
                                        on:click={
                                            let on_cancel = on_cancel.clone();
                                            move |ev| on_cancel(ev)
                                        }
                                        class="px-3 py-1 text-sm text-text-secondary \
                                               hover:text-text-primary transition-colors"
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        on:click={
                                            let on_save = on_save.clone();
                                            move |ev| on_save(ev)
                                        }
                                        class="px-3 py-1 text-sm bg-accent \
                                               text-on-accent rounded hover:bg-accent-hover \
                                               transition-colors"
                                    >
                                        "Add task"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                }
            }
        </Show>
    }
}

#[component]
pub fn EditTaskForm<S, C>(
    title: String,
    body: Option<String>,
    on_save: S,
    on_cancel: C,
) -> impl IntoView
where
    S: Fn(String, Option<String>) + Send + Sync + 'static,
    C: Fn() + Send + Sync + Clone + 'static,
{
    let (title_sig, set_title) = signal(title);
    let (body_sig, set_body) = signal(body.unwrap_or_default());
    let (preview, set_preview) = signal(false);
    let on_save = std::sync::Arc::new(on_save);

    let handle_save = {
        let on_save = on_save.clone();
        move |_| {
            let t = title_sig.get_untracked().trim().to_string();
            if !t.is_empty() {
                let b = body_sig.get_untracked().trim().to_string();
                let body_opt = if b.is_empty() { None } else { Some(b) };
                on_save(t, body_opt);
            }
        }
    };

    let handle_keydown = {
        let on_save = on_save.clone();
        let on_cancel = on_cancel.clone();
        std::sync::Arc::new(move |ev: KeyboardEvent| {
            if ev.key() == "Enter" && ev.shift_key() {
                ev.prevent_default();
                let t = title_sig.get_untracked().trim().to_string();
                if !t.is_empty() {
                    let b = body_sig.get_untracked().trim().to_string();
                    let body_opt = if b.is_empty() { None } else { Some(b) };
                    on_save(t, body_opt);
                }
            } else if ev.key() == "Escape" {
                on_cancel();
            }
        })
    };

    view! {
        <div class="border border-border rounded-xl p-3 shadow-sm \
                    focus-within:border-accent transition-colors">
            <AutocompleteInput
                value=title_sig
                set_value=set_title
                class="w-full text-sm font-semibold bg-transparent \
                       outline-none placeholder:text-text-secondary \
                       text-text-primary mb-2"
                on_keydown=handle_keydown
            />

            <Show
                when=move || preview.get()
                fallback=move || {
                    view! {
                        <AutocompleteTextarea
                            value=body_sig
                            set_value=set_body
                            placeholder="Description (markdown supported)"
                            rows=3
                            class="w-full text-sm bg-transparent outline-none \
                                   placeholder:text-text-tertiary \
                                   text-text-secondary resize-none"
                        />
                    }
                }
            >
                {move || {
                    let b = body_sig.get();
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
                        on:click=move |_| on_cancel()
                        class="px-3 py-1 text-sm text-text-secondary \
                               hover:text-text-primary transition-colors"
                    >
                        "Cancel"
                    </button>
                    <button
                        on:click=handle_save
                        class="px-3 py-1 text-sm bg-accent text-on-accent \
                               rounded hover:bg-accent-hover \
                               transition-colors"
                    >
                        "Save"
                    </button>
                </div>
            </div>
        </div>
    }
}
