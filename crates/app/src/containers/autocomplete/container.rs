use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;

use north_stores::AppStore;

use super::view::{AutocompleteInputView, AutocompleteTextareaView};

#[component]
pub fn AutocompleteInput(
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] class: &'static str,
    #[prop(optional)] on_keydown: Option<std::sync::Arc<dyn Fn(KeyboardEvent) + Send + Sync>>,
    #[prop(optional)] autofocus: bool,
    #[prop(optional)] on_blur: Option<Callback<()>>,
    #[prop(optional)] node_ref: Option<NodeRef<html::Input>>,
) -> impl IntoView {
    let app_store = use_context::<AppStore>();
    let tags = Signal::derive(move || app_store.map(|s| s.tags.get()).unwrap_or_default());
    let projects = Signal::derive(move || app_store.map(|s| s.projects.get()).unwrap_or_default());

    view! {
        <AutocompleteInputView
            value=value
            set_value=set_value
            placeholder=placeholder
            class=class
            on_keydown=on_keydown
            autofocus=autofocus
            on_blur=on_blur
            node_ref=node_ref
            tags=tags
            projects=projects
        />
    }
}

#[component]
pub fn AutocompleteTextarea(
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] class: &'static str,
    #[prop(optional, default = 3)] rows: u32,
    #[prop(optional)] on_keydown: Option<std::sync::Arc<dyn Fn(KeyboardEvent) + Send + Sync>>,
    #[prop(optional)] on_blur: Option<Callback<()>>,
    #[prop(optional)] on_input: Option<Callback<leptos::ev::Event>>,
    #[prop(optional)] node_ref: Option<NodeRef<html::Textarea>>,
    #[prop(optional)] autofocus: bool,
) -> impl IntoView {
    let app_store = use_context::<AppStore>();
    let tags = Signal::derive(move || app_store.map(|s| s.tags.get()).unwrap_or_default());
    let projects = Signal::derive(move || app_store.map(|s| s.projects.get()).unwrap_or_default());

    let textarea_ref = node_ref.unwrap_or_default();

    view! {
        <AutocompleteTextareaView
            value=value
            set_value=set_value
            placeholder=placeholder
            class=class
            rows=rows
            on_keydown=on_keydown
            on_blur=on_blur
            on_input=on_input
            node_ref=textarea_ref
            autofocus=autofocus
            tags=tags
            projects=projects
        />
    }
}
