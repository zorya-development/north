use leptos::prelude::*;
use pulldown_cmark::{html, Options, Parser};

pub fn render_markdown(input: &str) -> String {
    let options =
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    ammonia::clean(&html_output)
}

#[component]
pub fn MarkdownView(content: String) -> impl IntoView {
    let html = render_markdown(&content);

    view! {
        <div class="markdown-body" inner_html=html/>
    }
}
