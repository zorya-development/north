use leptos::prelude::*;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant, Toolbar, ToolbarSeparator};
use crate::components::keybindings_modal::KeybindingsModal;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::components::{SearchInput, TagFilterRow};
use crate::containers::traversable_task_list::{TraversableTaskList, TtlHandle};

#[component]
pub fn ReviewView(
    review_task_ids: Memo<Vec<i64>>,
    reviewed_task_ids: Memo<Vec<i64>>,
    is_loaded: Signal<bool>,
    hide_non_actionable: Signal<bool>,
    actionable_count: Memo<usize>,
    pending_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    reviewed_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    show_reviewed: ReadSignal<bool>,
    set_show_reviewed: WriteSignal<bool>,
    on_task_click: Callback<i64>,
    on_toggle_visibility: Callback<()>,
) -> impl IntoView {
    let review_config = ItemConfig {
        show_review: true,
        ..Default::default()
    };

    let show_keybindings_help = RwSignal::new(false);
    let (help_read, help_write) = show_keybindings_help.split();
    let ttl_handle = RwSignal::new(None::<TtlHandle>);

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
                    <Text variant=TextVariant::HeadingLg>"Review"</Text>
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
                root_task_ids=review_task_ids
                node_filter=pending_filter
                item_config=review_config
                is_loaded=is_loaded
                allow_create=false
                allow_reorder=false
                on_task_click=on_task_click
                show_keybindings_help=show_keybindings_help
                handle=ttl_handle
                search_query=search_query
                active_tag_names=active_tag_names
                empty_message="All tasks are up to date. Nothing to review."
            />

            <div class="border-t border-border pt-4">
                <button
                    data-testid="review-toggle-recent"
                    on:click=move |_| {
                        set_show_reviewed.update(|v| *v = !*v);
                    }
                    class="text-sm text-text-secondary \
                           hover:text-text-primary transition-colors"
                >
                    {move || {
                        if show_reviewed.get() {
                            "Hide recently reviewed"
                        } else {
                            "Show recently reviewed"
                        }
                    }}
                </button>
                <Show when=move || show_reviewed.get()>
                    <div class="mt-3">
                        <TraversableTaskList
                            root_task_ids=reviewed_task_ids
                            node_filter=reviewed_filter
                            item_config=review_config
                            is_loaded=is_loaded
                            allow_create=false
                            allow_reorder=false
                            on_task_click=on_task_click
                            empty_message="No recently reviewed tasks."
                        />
                    </div>
                </Show>
            </div>

            <KeybindingsModal open=help_read set_open=help_write />
        </div>
    }
}
