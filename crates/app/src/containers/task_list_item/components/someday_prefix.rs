use leptos::prelude::*;

#[component]
pub fn SomedayPrefix() -> impl IntoView {
    view! {
        <span class="text-sm font-medium mr-1 text-text-tertiary">
            "@"
            <a
                href="/someday"
                class="hover:underline"
                on:click=move |ev: leptos::ev::MouseEvent| {
                    ev.stop_propagation();
                }
            >
                "Someday"
            </a>
        </span>
    }
}
