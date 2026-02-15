use leptos::prelude::*;

#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center flex-1 min-h-[50vh]">
            <div
                class="w-8 h-8 rounded-full animate-spin"
                style="border: 3px solid var(--accent); border-top-color: transparent;"
            ></div>
        </div>
    }
}
