use leptos::prelude::*;

use crate::components::page_header::PageHeader;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::{ToolbarConfig, TraversableTaskList};
use crate::libs::TaskTreeView;

#[component]
pub fn TodayView(
    view: TaskTreeView,
    on_task_click: Callback<i64>,
    toolbar: ToolbarConfig,
    show_keybindings_help: RwSignal<bool>,
) -> impl IntoView {
    let item_config = ItemConfig::default();

    view! {
        <div class="space-y-4">
            <PageHeader title="Today" show_keybindings_help=show_keybindings_help />

            <TraversableTaskList
                view=view
                item_config=item_config
                allow_create=false
                allow_reorder=false
                on_task_click=on_task_click
                fold_storage_key="north:collapsed:today".to_string()
                toolbar=toolbar
                show_keybindings_help=show_keybindings_help
                empty_message="No tasks scheduled for today."
            />
        </div>
    }
}
