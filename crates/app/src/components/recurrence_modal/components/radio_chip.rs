use leptos::prelude::*;

#[component]
pub fn RadioChip(
    label: &'static str,
    active: Signal<bool>,
    on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click.run(())
            class=move || {
                if active.get() {
                    "px-3 py-1.5 rounded-full text-sm font-medium \
                     bg-accent text-on-accent transition-colors"
                } else {
                    "px-3 py-1.5 rounded-full text-sm font-medium \
                     bg-bg-input text-text-secondary \
                     border border-border \
                     hover:border-accent hover:text-text-primary \
                     transition-colors"
                }
            }
        >
            {label}
        </button>
    }
}
