use leptos::prelude::*;

#[component]
pub fn Modal(
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    #[prop(default = "md")] size: &'static str,
    children: Children,
) -> impl IntoView {
    let panel = children();

    let size_class = match size {
        "lg" => "max-w-3xl",
        "xl" => "max-w-5xl",
        _ => "max-w-md",
    };

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center"
            style:display=move || {
                if open.get() { "flex" } else { "none" }
            }
        >
            <div
                class="absolute inset-0 bg-backdrop"
                on:click=move |_| set_open.set(false)
            />
            <div class=format!(
                "relative z-10 bg-bg-secondary border border-border/60 \
                 rounded-2xl shadow-2xl {size_class} w-full mx-4"
            )>
                {panel}
            </div>
        </div>
    }
}
