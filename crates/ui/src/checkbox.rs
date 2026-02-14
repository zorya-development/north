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
                        <div class="w-4 h-4 rounded-full border \
                                    border-text-tertiary \
                                    hover:border-accent \
                                    hover:bg-accent/10 \
                                    transition-all duration-200" />
                    }
                }
            >
                <div class="w-4 h-4 rounded-full bg-accent \
                            hover:bg-accent-hover flex items-center \
                            justify-center transition-all duration-200">
                    <Icon kind=IconKind::Check class="w-3 h-3 text-on-accent"/>
                </div>
            </Show>
        </button>
    }
}
