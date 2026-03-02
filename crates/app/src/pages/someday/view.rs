use leptos::prelude::*;

use crate::components::page_header::PageHeader;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::{ToolbarConfig, TraversableTaskList};

#[component]
pub fn SomedayView(
    root_task_ids: Memo<Vec<i64>>,
    is_loaded: Signal<bool>,
    node_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    on_task_click: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    toolbar: ToolbarConfig,
    show_keybindings_help: RwSignal<bool>,
) -> impl IntoView {
    let item_config = ItemConfig {
        show_inline_project: true,
        show_someday: false,
        draggable: true,
        ..Default::default()
    };

    view! {
        <div class="space-y-4">
            <PageHeader title="Someday" show_keybindings_help=show_keybindings_help />

            <TraversableTaskList
                root_task_ids=root_task_ids
                node_filter=node_filter
                item_config=item_config
                is_loaded=is_loaded
                on_reorder=on_reorder
                on_task_click=on_task_click
                toolbar=toolbar
                show_keybindings_help=show_keybindings_help
                empty_message="No someday tasks. Press S on any task to defer it."
            />
        </div>
    }
}
