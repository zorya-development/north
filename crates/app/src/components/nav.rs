use leptos::prelude::*;
use leptos_router::hooks::use_location;

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

#[derive(Clone, Copy)]
enum IconKind {
    Inbox,
    Today,
    Tasks,
    Review,
    Filter,
    Stats,
    Settings,
}

#[component]
fn NavIcon(kind: IconKind) -> impl IntoView {
    let cls = "w-4 h-4 flex-shrink-0";
    match kind {
        IconKind::Inbox => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=cls viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/>
                <path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 \
                         2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 \
                         0 0 0-1.79 1.11z"/>
            </svg>
        }.into_any(),
        IconKind::Today => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=cls viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
            </svg>
        }.into_any(),
        IconKind::Tasks => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=cls viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 2 2 7l10 5 10-5-10-5z"/>
                <path d="m2 17 10 5 10-5"/>
                <path d="m2 12 10 5 10-5"/>
            </svg>
        }.into_any(),
        IconKind::Review => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=cls viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="1 4 1 10 7 10"/>
                <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
            </svg>
        }.into_any(),
        IconKind::Filter => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=cls viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
            </svg>
        }.into_any(),
        IconKind::Stats => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=cls viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="20" x2="18" y2="10"/>
                <line x1="12" y1="20" x2="12" y2="4"/>
                <line x1="6" y1="20" x2="6" y2="14"/>
            </svg>
        }.into_any(),
        IconKind::Settings => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=cls viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 \
                         0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 \
                         1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 \
                         2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 \
                         9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 \
                         1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 \
                         0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 \
                         1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 \
                         9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 \
                         0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 \
                         0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 \
                         1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 \
                         1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 \
                         1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 \
                         0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 \
                         0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 \
                         0-1.51 1z"/>
            </svg>
        }.into_any(),
    }
}

#[component]
fn NavItem(
    href: &'static str,
    label: &'static str,
    icon: IconKind,
) -> impl IntoView {
    let location = use_location();

    let is_active = move || {
        location.pathname.get() == href
    };

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
            <NavIcon kind=icon/>
            {label}
        </a>
    }
}
