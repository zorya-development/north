use leptos::prelude::*;

#[component]
pub fn Toolbar(#[prop(default = "")] class: &'static str, children: Children) -> impl IntoView {
    let classes = format!("flex flex-wrap items-center gap-x-3 gap-y-1 {class}");
    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

#[component]
pub fn ToolbarSeparator() -> impl IntoView {
    view! {
        <span class="text-text-tertiary text-xs">"|"</span>
    }
}
