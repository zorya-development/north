use leptos::prelude::*;
use north_domain::TagInfo;
use north_stores::use_app_store;

use super::view::TagPickerView;

#[component]
pub fn TagPicker(
    task_id: i64,
    tags: Vec<TagInfo>,
    on_set_tags: Callback<(i64, Vec<String>)>,
    #[prop(default = false)] icon_only: bool,
    #[prop(default = false)] always_visible: bool,
) -> impl IntoView {
    let (popover_open, set_popover_open) = signal(false);
    let current_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
    let (current_tags, set_current_tags) = signal(current_names);
    let (display_tags, set_display_tags) = signal(tags.clone());

    let app_store = use_app_store();
    let all_tags = Memo::new(move |_| app_store.tags.get());

    let wrapped_on_set_tags = Callback::new(move |args: (i64, Vec<String>)| {
        on_set_tags.run(args);
        app_store.tags.refetch();
    });

    view! {
        <TagPickerView
            task_id=task_id
            display_tags=display_tags
            set_display_tags=set_display_tags
            popover_open=popover_open
            set_popover_open=set_popover_open
            all_tags=all_tags
            current_tags=current_tags
            set_current_tags=set_current_tags
            on_set_tags=wrapped_on_set_tags
            icon_only=icon_only
            always_visible=always_visible
        />
    }
}
