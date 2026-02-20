use leptos::prelude::*;

#[component]
pub fn DayCheckbox(
    label: &'static str,
    selected: Signal<bool>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_toggle.run(())
            class=move || {
                if selected.get() {
                    "w-8 h-8 rounded-md text-xs font-medium \
                     bg-accent text-on-accent transition-colors"
                } else {
                    "w-8 h-8 rounded-md text-xs font-medium \
                     bg-bg-input text-text-tertiary \
                     border border-border \
                     hover:border-accent hover:text-text-secondary \
                     transition-colors"
                }
            }
        >
            {label}
        </button>
    }
}
