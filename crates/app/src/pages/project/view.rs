use leptos::prelude::*;
use north_dto::Project;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant, Toolbar, ToolbarSeparator};
use crate::components::keybindings_modal::KeybindingsModal;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::components::{SearchInput, TagFilterRow};
use crate::containers::traversable_task_list::{TraversableTaskList, TtlHandle};

#[component]
pub fn ProjectView(
    project: Memo<Option<Project>>,
    root_task_ids: Memo<Vec<i64>>,
    show_completed: RwSignal<bool>,
    completed_count: Memo<usize>,
    is_loaded: Signal<bool>,
    hide_non_actionable: Signal<bool>,
    actionable_count: Memo<usize>,
    node_filter: Signal<Callback<north_stores::TaskModel, bool>>,
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
                    <div data-testid="project-title">
                        <Text variant=TextVariant::HeadingLg>
                            {move || {
                                project
                                    .get()
                                    .map(|p| p.title)
                                    .unwrap_or_else(|| "Project".to_string())
                            }}
                        </Text>
                    </div>
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
                            data-testid="project-add-task"
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
                        <ToolbarSeparator />
                        <button
                            on:click=move |_| {
                                show_completed.update(|v| *v = !*v)
                            }
                            class=move || {
                                if show_completed.get() {
                                    "text-xs text-accent cursor-pointer transition-colors"
                                } else {
                                    "text-xs text-text-secondary hover:text-text-primary \
                                     cursor-pointer transition-colors"
                                }
                            }
                        >
                            {move || format!("Completed ({})", completed_count.get())}
                        </button>
                        <ToolbarSeparator />
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
                default_project_id=default_project_id
                handle=ttl_handle
                search_query=search_query
                active_tag_names=active_tag_names
                empty_message="No tasks in this project. Add one above."
            />

            <KeybindingsModal open=help_read set_open=help_write />
        </div>
    }
}
