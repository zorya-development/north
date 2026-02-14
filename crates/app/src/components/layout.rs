use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use north_stores::AppStore;

use crate::components::nav::Sidebar;
use crate::server_fns::auth::check_auth;
use crate::stores::lookup_store::LookupStore;

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let auth_check = Resource::new(|| (), |_| check_auth());
    let navigate = use_navigate();

    let app_store = AppStore::new();
    provide_context(app_store);

    provide_context(LookupStore::new());

    Effect::new(move || {
        if let Some(Err(_)) = auth_check.get() {
            navigate("/login", Default::default());
        }
    });

    Effect::new(move || {
        app_store.projects.refetch();
    });

    view! {
        <div class="flex h-screen">
            <Sidebar/>
            <main class="flex-1 overflow-y-auto bg-bg-primary">
                <div class="max-w-4xl mx-auto px-8 py-10">{children()}</div>
            </main>
        </div>
    }
}
