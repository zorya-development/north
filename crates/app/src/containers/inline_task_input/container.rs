use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::CreateTask;
use north_stores::use_app_store;
use wasm_bindgen::JsCast;

use super::view::InlineTaskInputView;

#[component]
pub fn InlineTaskInput(
    parent_id: i64,
    value: RwSignal<String>,
    on_created: Callback<i64>,
    on_close: Callback<()>,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let app_store = use_app_store();
    let parent_task = app_store.tasks.get_by_id(parent_id);
    let input_ref = NodeRef::<leptos::html::Textarea>::new();

    Effect::new(move || {
        if let Some(el) = input_ref.get() {
            let _ = el.focus();
            let len = value.get_untracked().len() as u32;
            let _ = el.set_selection_range(len, len);
        }
    });

    view! {
        <InlineTaskInputView
            value=value
            input_ref=input_ref
            on_submit=Callback::new(move |(title, body): (String, Option<String>)| {
                let project_id = parent_task
                    .get_untracked()
                    .and_then(|t| t.project_id);
                let input = CreateTask {
                    title,
                    body,
                    parent_id: Some(parent_id),
                    project_id,
                    ..Default::default()
                };
                value.set(String::new());
                let store = app_store.tasks;
                spawn_local(async move {
                    if let Some(task) =
                        store.create_task_async(input).await
                    {
                        on_created.run(task.id);
                    }
                    if let Some(el) = input_ref.get() {
                        let _ = el.focus();
                        // Reset height after clearing
                        if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                            let _ = html_el.style().set_property("height", "auto");
                        }
                    }
                });
            })
            on_close=on_close
            class=class
        />
    }
}
