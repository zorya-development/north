use leptos::prelude::*;

use super::view::ProjectPickerView;
use crate::server_fns::projects::get_projects;

#[component]
pub fn ProjectPicker(
    task_id: i64,
    project_id: Option<i64>,
    project_title: Option<String>,
    on_set_project: Callback<(i64, i64)>,
    on_clear_project: Callback<i64>,
    #[prop(default = false)] icon_only: bool,
) -> impl IntoView {
    let has_project = project_id.is_some();
    let (popover_open, set_popover_open) = signal(false);

    let projects = Resource::new(
        move || popover_open.get(),
        move |open| async move {
            if open {
                get_projects().await
            } else {
                Ok(vec![])
            }
        },
    );

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
        />
    }
}
