use leptos::prelude::*;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant};
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::{ToolbarConfig, TraversableTaskList};

#[component]
pub fn AllTasksView(
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
        draggable: true,
        ..Default::default()
    };

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <Text variant=TextVariant::HeadingLg>"All Tasks"</Text>
                <button
                    on:click=move |_| show_keybindings_help.set(true)
                    class="flex items-center gap-1.5 text-xs \
                           text-text-secondary hover:text-text-primary \
                           transition-colors cursor-pointer"
                    title="Keyboard shortcuts"
                >
                    <Icon kind=IconKind::Keyboard class="w-3.5 h-3.5" />
                    <span class="font-mono">"?"</span>
                    " for help"
                </button>
            </div>

            <TraversableTaskList
                root_task_ids=root_task_ids
                node_filter=node_filter
                item_config=item_config
                is_loaded=is_loaded
                on_reorder=on_reorder
                on_task_click=on_task_click
                toolbar=toolbar
                show_keybindings_help=show_keybindings_help
                empty_message="No tasks yet. Add one above."
            />
        </div>
    }
}
