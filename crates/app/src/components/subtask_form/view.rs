use leptos::prelude::*;
use north_ui::{Icon, IconKind};

#[component]
pub fn SubtaskFormView(
    expanded: ReadSignal<bool>,
    set_expanded: WriteSignal<bool>,
    title: ReadSignal<String>,
    set_title: WriteSignal<String>,
    depth: usize,
    on_submit: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move || {
        if expanded.get() {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        }
    });

    let pad_fallback = format!("pl-{}", depth * 4);
    let pad_expanded = pad_fallback.clone();

    view! {
        <Show
            when=move || expanded.get()
            fallback=move || {
                let pad = pad_fallback.clone();
                view! {
                    <button
                        class=format!(
                            "flex items-center gap-1 text-xs \
                             text-text-tertiary hover:text-text-secondary \
                             transition-colors py-1 {pad}",
                        )
                        on:click=move |_| set_expanded.set(true)
                    >
                        <Icon kind=IconKind::Plus class="w-3 h-3"/>
                        "Add sub-task"
                    </button>
                }
            }
        >
            <div class=format!("flex items-center gap-1 py-0.5 {pad_expanded}")>
                <input
                    type="text"
                    node_ref=input_ref
                    class="flex-1 text-sm bg-bg-input border border-border \
                           rounded px-2 py-0.5 text-text-primary \
                           placeholder:text-text-tertiary \
                           focus:outline-none focus:border-accent"
                    placeholder="Sub-task title..."
                    prop:value=move || title.get()
                    on:input=move |ev| {
                        set_title.set(event_target_value(&ev));
                    }
                    on:keydown=move |ev| {
                        match ev.key().as_str() {
                            "Enter" => {
                                ev.prevent_default();
                                on_submit.run(());
                            }
                            "Escape" => {
                                on_cancel.run(());
                            }
                            _ => {}
                        }
                    }
                />
            </div>
        </Show>
    }
}
