use leptos::prelude::*;
use north_dto::Project;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant};
use crate::components::keybindings_modal::KeybindingsModal;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::{TraversableTaskList, TtlHandle};

#[component]
pub fn ProjectView(
    project: Memo<Option<Project>>,
    root_task_ids: Memo<Vec<i64>>,
    show_completed: RwSignal<bool>,
    completed_count: Memo<usize>,
    is_loaded: Signal<bool>,
    hide_non_actionable: Signal<bool>,
    node_filter: Callback<north_stores::TaskModel, bool>,
    default_project_id: Signal<Option<i64>>,
    on_task_click: Callback<i64>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    on_toggle_visibility: Callback<()>,
) -> impl IntoView {
    let show_keybindings_help = RwSignal::new(false);
    let (help_read, help_write) = show_keybindings_help.split();
    let ttl_handle = RwSignal::new(None::<TtlHandle>);
    let item_config = ItemConfig {
        show_project: false,
        draggable: true,
        ..Default::default()
    };

    view! {
        <div class="space-y-4">
            <div>
                <div class="flex items-center justify-between">
                    <Text variant=TextVariant::HeadingLg>
                        {move || {
                            project
                                .get()
                                .map(|p| p.title)
                                .unwrap_or_else(|| "Project".to_string())
                        }}
                    </Text>
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
                <div class="flex items-center gap-3 mt-2">
                    <button
                        on:click=move |_| {
                            if let Some(h) = ttl_handle.get_untracked() {
                                h.start_create_top();
                            }
                        }
                        class="text-xs text-text-secondary hover:text-text-primary \
                               transition-colors cursor-pointer"
                    >
                        "+" " Add task"
                    </button>
                    {move || {
                        let count = completed_count.get();
                        if count > 0 {
                            Some(
                                view! {
                                    <button
                                        on:click=move |_| {
                                            show_completed.update(|v| *v = !*v)
                                        }
                                        class="text-xs text-text-secondary \
                                               hover:text-text-primary \
                                               transition-colors cursor-pointer"
                                    >
                                        {move || {
                                            if show_completed.get() {
                                                format!(
                                                    "Hide completed ({count})",
                                                )
                                            } else {
                                                format!(
                                                    "Show completed ({count})",
                                                )
                                            }
                                        }}
                                    </button>
                                },
                            )
                        } else {
                            None
                        }
                    }}
                    <button
                        on:click=move |_| on_toggle_visibility.run(())
                        class="text-xs text-text-secondary \
                               hover:text-text-primary transition-colors \
                               cursor-pointer"
                    >
                        {move || {
                            if hide_non_actionable.get() {
                                "Show all tasks"
                            } else {
                                "Hide non-actionable"
                            }
                        }}
                    </button>
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
                default_project_id=default_project_id
                handle=ttl_handle
                empty_message="No tasks in this project. Add one above."
            />

            <KeybindingsModal open=help_read set_open=help_write />
        </div>
    }
}
