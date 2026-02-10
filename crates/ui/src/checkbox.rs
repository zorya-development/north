use leptos::prelude::*;

use crate::icon::{Icon, IconKind};

#[component]
pub fn Checkbox(
    checked: ReadSignal<bool>,
    on_toggle: Callback<()>,
    #[prop(optional, default = "Uncheck")] checked_label: &'static str,
    #[prop(optional, default = "Check")] unchecked_label: &'static str,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_toggle.run(())
            class="flex-shrink-0"
            aria-label=move || {
                if checked.get() {
                    checked_label
                } else {
                    unchecked_label
                }
            }
        >
            <Show
                when=move || checked.get()
                fallback=move || {
                    view! {
                        <div class="w-4 h-4 rounded-full border-2 \
                                    border-text-secondary \
                                    hover:border-accent \
                                    hover:bg-accent \
                                    transition-colors" />
                    }
                }
            >
                <div class="w-4 h-4 rounded-full bg-text-tertiary \
                            hover:bg-text-secondary flex items-center \
                            justify-center transition-colors">
                    <Icon kind=IconKind::Check class="w-2.5 h-2.5 text-bg-primary"/>
                </div>
            </Show>
        </button>
    }
}
