use leptos::prelude::*;
use north_ui::MarkdownView;

#[component]
pub fn EnrichedMarkdownViewInner(content: String) -> impl IntoView {
    view! {
        <MarkdownView content=content/>
    }
}
