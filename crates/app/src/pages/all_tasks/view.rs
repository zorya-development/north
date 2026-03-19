use leptos::prelude::*;

use crate::components::page_header::PageHeader;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::{ToolbarConfig, TraversableTaskList};
use crate::controllers::TaskTreeView;

#[component]
pub fn AllTasksView(
    view: TaskTreeView,
    on_task_click: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    toolbar: ToolbarConfig,
    show_keybindings_help: RwSignal<bool>,
) -> impl IntoView {
    let item_config = ItemConfig {
        show_inline_project: true,
        draggable: true,
        ..Default::default()
    };

    view! {
        <div class="space-y-4">
            <PageHeader title="All Tasks" show_keybindings_help=show_keybindings_help />

            <TraversableTaskList
                view=view
                item_config=item_config
                on_reorder=on_reorder
                on_task_click=on_task_click
                toolbar=toolbar
                show_keybindings_help=show_keybindings_help
                empty_message="No tasks yet. Add one above."
            />
        </div>
    }
}
