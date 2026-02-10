use leptos::prelude::*;
use north_domain::TagInfo;

use super::view::TagPickerView;
use crate::server_fns::tags::get_tags;

#[component]
pub fn TagPicker(
    task_id: i64,
    tags: Vec<TagInfo>,
    on_set_tags: Callback<(i64, Vec<String>)>,
) -> impl IntoView {
    let (popover_open, set_popover_open) = signal(false);
    let current_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
    let (current_tags, set_current_tags) = signal(current_names);

    let all_tags = Resource::new(
        move || popover_open.get(),
        move |open| async move {
            if open {
                get_tags().await
            } else {
                Ok(vec![])
            }
        },
    );

    view! {
        <TagPickerView
            task_id=task_id
            tags=tags
            popover_open=popover_open
            set_popover_open=set_popover_open
            all_tags=all_tags
            current_tags=current_tags
            set_current_tags=set_current_tags
            on_set_tags=on_set_tags
        />
    }
}
