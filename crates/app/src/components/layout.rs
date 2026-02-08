use leptos::prelude::*;

use crate::components::nav::Sidebar;

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    view! {
        <div class="flex h-screen">
            <Sidebar/>
            <main class="flex-1 overflow-y-auto">
                <div class="max-w-5xl mx-auto px-6 py-8">{children()}</div>
            </main>
        </div>
    }
}
