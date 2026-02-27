use leptos::prelude::*;

use north_stores::AppStore;

use super::view::SmartTextareaView;

/// Unified textarea composing autocomplete, mirror overlay, auto-resize,
/// multiline (Ctrl+Enter), and strip-newlines via props.
///
/// Container pulls tags/projects from AppStore for autocomplete;
/// the view is pure rendering.
#[component]
pub fn SmartTextarea(
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] class: &'static str,
    #[prop(optional)] node_ref: Option<NodeRef<leptos::html::Textarea>>,
    #[prop(optional)] data_testid: &'static str,

    // Feature toggles
    #[prop(optional)] autocomplete: bool,
    #[prop(optional)] mirror_overlay: bool,
    #[prop(optional)] auto_resize: bool,
    #[prop(optional)] multiline: bool,
    #[prop(optional)] strip_newlines: bool,

    // Behavior callbacks
    #[prop(optional)] on_submit: Option<Callback<()>>,
    #[prop(optional)] on_close: Option<Callback<()>>,
    #[prop(optional)] on_blur: Option<Callback<()>>,
    #[prop(optional)] on_input: Option<Callback<()>>,
    #[prop(optional)] autofocus: bool,
    #[prop(optional, default = 1)] rows: u32,
) -> impl IntoView {
    let app_store = use_context::<AppStore>();
    let tags = Signal::derive(move || app_store.map(|s| s.tags.get()).unwrap_or_default());
    let projects = Signal::derive(move || app_store.map(|s| s.projects.get()).unwrap_or_default());

    view! {
        <SmartTextareaView
            value=value
            placeholder=placeholder
            class=class
            node_ref=node_ref
            data_testid=data_testid
            autocomplete=autocomplete
            mirror_overlay=mirror_overlay
            auto_resize=auto_resize
            multiline=multiline
            strip_newlines=strip_newlines
            on_submit=on_submit
            on_close=on_close
            on_blur=on_blur
            on_input=on_input
            autofocus=autofocus
            rows=rows
            tags=tags
            projects=projects
        />
    }
}
