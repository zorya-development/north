use leptos::prelude::*;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant};

#[component]
pub fn PageHeader(title: &'static str, show_keybindings_help: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between">
            <Text variant=TextVariant::HeadingLg>{title}</Text>
            <button
                on:click=move |_| show_keybindings_help.set(true)
                class="flex items-center gap-1.5 text-xs \
                       text-text-secondary hover:text-text-primary \
                       transition-colors cursor-pointer"
                title="Keyboard shortcuts"
            >
                <Icon kind=IconKind::Keyboard class="w-3.5 h-3.5" />
                <span class="font-mono">"?"</span>
                " for help"
            </button>
        </div>
    }
}
