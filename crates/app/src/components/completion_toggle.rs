use leptos::prelude::*;

use crate::components::icons::{Icon, IconKind};

#[component]
pub fn CompletionToggle(
    is_completed: ReadSignal<bool>,
    on_toggle: std::sync::Arc<dyn Fn() + Send + Sync>,
) -> impl IntoView {
    view! {
        <button
            on:click={
                let on_toggle = on_toggle.clone();
                move |_| on_toggle()
            }
            class="flex-shrink-0"
            aria-label=move || {
                if is_completed.get() {
                    "Mark task incomplete"
                } else {
                    "Complete task"
                }
            }
        >
            <Show
                when=move || is_completed.get()
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
