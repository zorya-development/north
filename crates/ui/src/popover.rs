use leptos::prelude::*;

#[component]
pub fn Popover(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    trigger: Children,
    children: Children,
) -> impl IntoView {
    let panel = children();

    view! {
        <div class="relative inline-flex">
            {trigger()}
            <div
                class="fixed inset-0 z-40"
                style:display=move || {
                    if open.get() { "block" } else { "none" }
                }
                on:click=move |_| set_open.set(false)
            />
            <div
                class="absolute top-full left-0 mt-1 z-50 \
                        bg-bg-secondary border border-border \
                        rounded-lg shadow-lg"
                style:display=move || {
                    if open.get() { "block" } else { "none" }
                }
            >
                {panel}
            </div>
        </div>
    }
}
