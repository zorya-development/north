use leptos::prelude::*;
use north_stores::use_app_store;

use super::view::SidebarView;

#[component]
pub fn Sidebar() -> impl IntoView {
    let app_store = use_app_store();

    view! {
        <SidebarView
            projects=Signal::derive(move || app_store.projects.get())
            saved_filters=Signal::derive(move || app_store.saved_filters.get())
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
        />
    }
}
