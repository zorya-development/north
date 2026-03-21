use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};
use north_repositories::ErrorNotifier;
use north_stores::status_bar_store::StatusBarVariant;
use north_stores::AppStore;
use north_ui::{Icon, IconKind};

use crate::components::connectivity_monitor::ConnectivityMonitor;
use crate::components::status_bar::StatusBar;
use crate::containers::sidebar::Sidebar;
use crate::containers::task_detail_modal::TaskDetailModal;
use north_server_fns::auth::check_auth;

/// Mobile sidebar open/close state — provided as context so sidebar can close
/// the drawer on navigation. Only relevant below the `lg` breakpoint.
#[derive(Clone, Copy)]
pub struct MobileSidebarState(pub RwSignal<bool>);

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let auth_check = Resource::new(|| (), |_| check_auth());
    let navigate = use_navigate();

    let app_store = AppStore::new();
    provide_context(app_store);
    provide_context(app_store.modal);
    provide_context(app_store.task_detail_modal);

    let mobile_sidebar_open = RwSignal::new(false);
    provide_context(MobileSidebarState(mobile_sidebar_open));

    let status_bar = app_store.status_bar;
    provide_context(ErrorNotifier(Callback::new(move |msg: String| {
        status_bar.notify(StatusBarVariant::Danger, msg);
    })));

    Effect::new(move || {
        if let Some(Err(_)) = auth_check.get() {
            navigate("/login", Default::default());
        }
    });

    Effect::new(move || {
        app_store.refetch();
    });

    // Auto-close mobile drawer on route change
    let location = use_location();
    Effect::new(move || {
        location.pathname.get();
        mobile_sidebar_open.set(false);
    });

    let sidebar_wrapper_class = move || {
        format!(
            "flex-shrink-0 \
             max-lg:fixed max-lg:inset-y-0 max-lg:left-0 max-lg:z-50 \
             {}",
            if mobile_sidebar_open.get() {
                "max-lg:block"
            } else {
                "max-lg:hidden"
            }
        )
    };

    let backdrop_class = move || {
        if mobile_sidebar_open.get() {
            "fixed inset-0 z-40 bg-backdrop transition-opacity duration-200 lg:hidden"
        } else {
            "fixed inset-0 z-40 bg-backdrop opacity-0 pointer-events-none \
             transition-opacity duration-200 lg:hidden"
        }
    };

    view! {
        <div class="flex h-screen">
            // Mobile header with hamburger (hidden on desktop)
            <div class="lg:hidden fixed top-0 left-0 right-0 z-30 \
                         flex items-center h-12 px-4 pt-safe \
                         bg-bg-primary border-b border-(--border-muted)">
                <button
                    on:click=move |_| mobile_sidebar_open.set(true)
                    class="p-2 -ml-2 rounded-lg \
                           text-text-secondary hover:text-text-primary \
                           hover:bg-bg-tertiary transition-colors"
                    aria-label="Open menu"
                >
                    <Icon kind=IconKind::Menu class="w-5 h-5"/>
                </button>
            </div>

            // Backdrop (mobile only)
            <div
                data-testid="mobile-sidebar-backdrop"
                class=backdrop_class
                on:click=move |_| mobile_sidebar_open.set(false)
            />

            // Sidebar wrapper: fixed overlay on mobile, static in-flow on desktop
            <div class=sidebar_wrapper_class>
                <Sidebar/>
            </div>

            // Main content (top padding on mobile for the header bar)
            <main class="flex-1 overflow-y-auto bg-bg-primary pt-12 lg:pt-0">
                <div class="max-w-4xl mx-auto px-6 py-6 lg:px-8 lg:py-10">
                    {children()}
                </div>
            </main>
        </div>
        <TaskDetailModal/>
        <ConnectivityMonitor/>
        <StatusBar/>
    }
}
