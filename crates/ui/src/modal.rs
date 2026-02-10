use leptos::prelude::*;

#[component]
pub fn Modal(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    children: Children,
) -> impl IntoView {
    let panel = children();

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center"
            style:display=move || {
                if open.get() { "flex" } else { "none" }
            }
        >
            <div
                class="absolute inset-0 bg-black/50"
                on:click=move |_| set_open.set(false)
            />
            <div class="relative z-10 bg-bg-secondary border border-border \
                        rounded-lg shadow-xl max-w-md w-full mx-4">
                {panel}
            </div>
        </div>
    }
}
