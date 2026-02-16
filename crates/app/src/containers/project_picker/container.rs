use leptos::prelude::*;
use north_stores::use_app_store;

use super::view::ProjectPickerView;

#[component]
pub fn ProjectPicker(
    task_id: i64,
    project_id: Option<i64>,
    project_title: Option<String>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    #[prop(default = false)] icon_only: bool,
    #[prop(default = false)] always_visible: bool,
) -> impl IntoView {
    let has_project = project_id.is_some();
    let (popover_open, set_popover_open) = signal(false);

    let app_store = use_app_store();
    let projects = Memo::new(move |_| app_store.projects.get());

    view! {
        <ProjectPickerView
            task_id=task_id
            has_project=has_project
            project_title=project_title
            popover_open=popover_open
            set_popover_open=set_popover_open
            projects=projects
            on_set_project=on_set_project
            on_clear_project=on_clear_project
            icon_only=icon_only
            always_visible=always_visible
        />
    }
}
