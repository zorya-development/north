use leptos::prelude::*;

use crate::components::page_header::PageHeader;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::{ToolbarConfig, TraversableTaskList};

#[component]
pub fn TodayView(
    root_task_ids: Memo<Vec<i64>>,
    is_loaded: Signal<bool>,
    node_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    on_task_click: Callback<i64>,
    toolbar: ToolbarConfig,
    show_keybindings_help: RwSignal<bool>,
) -> impl IntoView {
    let item_config = ItemConfig::default();

    view! {
        <div class="space-y-4">
            <PageHeader title="Today" show_keybindings_help=show_keybindings_help />

            <TraversableTaskList
                root_task_ids=root_task_ids
                node_filter=node_filter
                item_config=item_config
                is_loaded=is_loaded
                allow_create=false
                allow_reorder=false
                on_task_click=on_task_click
                toolbar=toolbar
                show_keybindings_help=show_keybindings_help
                empty_message="No tasks scheduled for today."
            />
        </div>
    }
}
