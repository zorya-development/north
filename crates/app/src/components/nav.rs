use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::icons::{Icon, IconKind};

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="w-56 bg-bg-secondary flex flex-col h-full">
            <div class="py-4 px-2 flex items-center gap-2">
                <img src="/public/logo.png" alt="North" class="w-10 h-10"/>
                <span class="text-lg font-semibold text-text-primary">"North"</span>
            </div>

            <nav class="flex-1 px-2 space-y-1">
                <NavItem href="/inbox" label="Inbox" icon=IconKind::Inbox/>
                <NavItem href="/today" label="Today" icon=IconKind::Today/>
                <NavItem href="/tasks" label="All Tasks" icon=IconKind::Tasks/>

                <div class="pt-4">
                    <span class="px-3 text-xs font-medium text-text-secondary \
                                 uppercase tracking-wide">
                        "Projects"
                    </span>
                </div>

                <div class="pt-4">
                    <NavItem href="/review" label="Review" icon=IconKind::Review/>
                    <NavItem href="/filter" label="Filters" icon=IconKind::Filter/>
                    <NavItem href="/stats" label="Stats" icon=IconKind::Stats/>
                </div>
            </nav>

            <div class="p-2 border-t border-border">
                <NavItem href="/settings" label="Settings" icon=IconKind::Settings/>
            </div>
        </aside>
    }
}

#[component]
fn NavItem(href: &'static str, label: &'static str, icon: IconKind) -> impl IntoView {
    let location = use_location();

    let is_active = move || location.pathname.get() == href;

    let class = move || {
        let base = "flex items-center gap-2 px-3 py-2 rounded-md text-sm \
                    text-text-primary hover:bg-bg-tertiary transition-colors";
        if is_active() {
            format!("{base} bg-bg-tertiary font-medium")
        } else {
            base.to_string()
        }
    };

    view! {
        <a href=href class=class>
            <Icon kind=icon class="w-4 h-4 flex-shrink-0"/>
            {label}
        </a>
    }
}
