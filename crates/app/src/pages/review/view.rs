use leptos::prelude::*;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant};
use crate::components::keybindings_modal::KeybindingsModal;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::TraversableTaskList;

#[component]
pub fn ReviewView(
    review_task_ids: Memo<Vec<i64>>,
    reviewed_task_ids: Memo<Vec<i64>>,
    is_loaded: Signal<bool>,
    hide_non_actionable: Signal<bool>,
    pending_filter: Callback<north_dto::Task, bool>,
    reviewed_filter: Callback<north_dto::Task, bool>,
    show_reviewed: ReadSignal<bool>,
    set_show_reviewed: WriteSignal<bool>,
    on_review_all: Callback<()>,
    on_task_click: Callback<i64>,
    on_toggle_visibility: Callback<()>,
) -> impl IntoView {
    let review_config = ItemConfig {
        show_review: true,
        ..Default::default()
    };

    let show_keybindings_help = RwSignal::new(false);
    let (help_read, help_write) = show_keybindings_help.split();

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
                <div class="flex items-center gap-3 mt-2">
                    <button
                        on:click=move |_| on_review_all.run(())
                        class="text-sm text-text-secondary hover:text-accent \
                               transition-colors cursor-pointer"
                    >
                        "Mark All as Reviewed"
                    </button>
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
                root_task_ids=review_task_ids
                node_filter=pending_filter
                item_config=review_config
                is_loaded=is_loaded
                allow_create=false
                allow_reorder=false
                on_task_click=on_task_click
                show_keybindings_help=show_keybindings_help
                empty_message="All tasks are up to date. Nothing to review."
            />

            <div class="border-t border-border pt-4">
                <button
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
