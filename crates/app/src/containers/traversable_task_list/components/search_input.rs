use leptos::prelude::*;
use north_ui::{Icon, IconKind};

#[component]
pub fn SearchInput(query: RwSignal<String>) -> impl IntoView {
    view! {
        <div class="flex items-center gap-1.5 text-text-tertiary \
                    px-2 py-1.5 lg:px-0 lg:py-0 \
                    bg-bg-input lg:bg-transparent \
                    border border-border lg:border-0 \
                    rounded lg:rounded-none">
            <Icon kind=IconKind::Search class="w-3.5 h-3.5 flex-shrink-0" />
            <input
                type="text"
                data-testid="task-search-input"
                placeholder="Search..."
                class="bg-transparent text-xs text-text-primary \
                       placeholder:text-text-tertiary \
                       outline-none no-focus-ring w-full"
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
            />
        </div>
    }
}
