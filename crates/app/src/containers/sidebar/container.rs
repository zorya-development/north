use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use north_stores::use_app_store;

use super::view::SidebarView;

#[component]
pub fn Sidebar() -> impl IntoView {
    let app_store = use_app_store();

    let collapsed = app_store.settings.sidebar_collapsed();

    let on_toggle_collapsed = Callback::new(move |_: ()| {
        app_store.settings.toggle_sidebar_collapsed();
    });

    // Cmd/Ctrl+B keyboard shortcut
    window_event_listener(leptos::ev::keydown, move |ev| {
        let key = ev.key();
        if key != "b" && key != "B" {
            return;
        }

        let meta = ev.meta_key() || ev.ctrl_key();
        if !meta {
            return;
        }

        // Skip when input/textarea is focused
        if let Some(el) = document().active_element() {
            if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                let tag = html_el.tag_name().to_lowercase();
                if tag == "input" || tag == "textarea" || html_el.is_content_editable() {
                    return;
                }
            }
        }

        // Skip when any modal is open
        if app_store.modal.is_any_open() {
            return;
        }

        ev.prevent_default();
        app_store.settings.toggle_sidebar_collapsed();
    });

    view! {
        <SidebarView
            projects=Signal::derive(move || app_store.projects.get())
            saved_filters=Signal::derive(move || app_store.saved_filters.get())
            collapsed=collapsed
            on_toggle_collapsed=on_toggle_collapsed
            on_create_project=Callback::new(move |title: String| {
                app_store.projects.create(title);
            })
            on_archive_project=Callback::new(move |id: i64| {
                app_store.projects.archive(id);
            })
            on_edit_project=Callback::new(move |(id, title, color): (i64, String, String)| {
                app_store.projects.update_details(id, title, color);
            })
            on_drop_task_to_project=Callback::new(
                move |(task_id, project_id): (i64, i64)| {
                    app_store.tasks.set_project(task_id, project_id);
                },
            )
            on_delete_filter=Callback::new(move |id: i64| {
                app_store.saved_filters.delete(id);
            })
        />
    }
}
