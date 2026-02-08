use leptos::prelude::*;

use crate::util::render_markdown;

#[component]
pub fn MarkdownView(content: String) -> impl IntoView {
    let html = render_markdown(&content);

    view! {
        <div class="markdown-body" inner_html=html/>
    }
}
