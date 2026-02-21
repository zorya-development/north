use leptos::prelude::*;
use north_stores::AppStore;

use super::controller::enrich_body;
use super::view::EnrichedMarkdownViewInner;

/// Renders markdown body text with enriched `#tag` and `@project` tokens.
/// - `#tag` → `[#tag](/filters/new?q=tags%3D%22tagname%22)` (link to filter page)
/// - `@project` → `[@project](/project/{id})` (link to project page)
/// Tokens inside code blocks and existing markdown links are left unchanged.
#[component]
pub fn EnrichedMarkdownView(content: String) -> impl IntoView {
    let app_store = use_context::<AppStore>();
    let enriched = enrich_body(&content, app_store);

    view! {
        <EnrichedMarkdownViewInner content=enriched/>
    }
}
