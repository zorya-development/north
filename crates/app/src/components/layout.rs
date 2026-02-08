use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::nav::Sidebar;
use crate::server_fns::auth::check_auth;

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let auth_check = Resource::new(|| (), |_| check_auth());
    let navigate = use_navigate();

    Effect::new(move || {
        if let Some(Err(_)) = auth_check.get() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <div class="flex h-screen">
            <Sidebar/>
            <main class="flex-1 overflow-y-auto bg-bg-primary">
                <div class="max-w-5xl mx-auto px-6 py-8">{children()}</div>
            </main>
        </div>
    }
}
