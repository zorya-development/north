use leptos::prelude::*;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant, Toolbar, ToolbarSeparator};
use crate::components::keybindings_modal::KeybindingsModal;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::components::{SearchInput, TagFilterRow};
use crate::containers::traversable_task_list::{TraversableTaskList, TtlHandle};

#[component]
pub fn SomedayView(
    root_task_ids: Memo<Vec<i64>>,
    is_loaded: Signal<bool>,
    hide_non_actionable: Signal<bool>,
    actionable_count: Memo<usize>,
    node_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    on_task_click: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    on_toggle_visibility: Callback<()>,
) -> impl IntoView {
    let show_keybindings_help = RwSignal::new(false);
    let (help_read, help_write) = show_keybindings_help.split();
    let ttl_handle = RwSignal::new(None::<TtlHandle>);
    let item_config = ItemConfig {
        show_inline_project: true,
        show_someday: false,
        draggable: true,
        ..Default::default()
    };

    let search_query = RwSignal::new(String::new());
    let active_tag_names: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let available_tags = Memo::new(move |_| {
        ttl_handle
            .get()
            .map(|h| h.available_tags().get())
            .unwrap_or_default()
    });

    view! {
        <div class="space-y-4">
            <div>
                <div class="flex items-center justify-between">
                    <Text variant=TextVariant::HeadingLg>"Someday"</Text>
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
                <div class="mt-2">
                    <Toolbar class="mb-2">
                        <button
                            on:click=move |_| on_toggle_visibility.run(())
                            class=move || {
                                if hide_non_actionable.get() {
                                    "text-xs text-accent cursor-pointer transition-colors"
                                } else {
                                    "text-xs text-text-secondary hover:text-text-primary \
                                     cursor-pointer transition-colors"
                                }
                            }
                        >
                            {move || format!("Actionable ({})", actionable_count.get())}
                        </button>
                        <ToolbarSeparator />
                        <SearchInput query=search_query />
                    </Toolbar>
                    <TagFilterRow
                        available_tags=available_tags
                        active_tag_names=active_tag_names
                        on_toggle=Callback::new(move |name: String| {
                            active_tag_names.update(|tags| {
                                if let Some(pos) = tags.iter().position(|t| *t == name) {
                                    tags.remove(pos);
                                } else {
                                    tags.push(name);
                                }
                            });
                        })
                    />
                </div>
            </div>

            <TraversableTaskList
                root_task_ids=root_task_ids
                node_filter=node_filter
                item_config=item_config
                is_loaded=is_loaded
                on_reorder=on_reorder
                on_task_click=on_task_click
                show_keybindings_help=show_keybindings_help
                handle=ttl_handle
                search_query=search_query
                active_tag_names=active_tag_names
                empty_message="No someday tasks. Press S on any task to defer it."
            />

            <KeybindingsModal open=help_read set_open=help_write />
        </div>
    }
}
