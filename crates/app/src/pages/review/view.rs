use leptos::prelude::*;

use crate::components::page_header::PageHeader;
use crate::containers::task_list_item::ItemConfig;
use crate::containers::traversable_task_list::{ToolbarConfig, TraversableTaskList};

#[component]
pub fn ReviewView(
    review_task_ids: Memo<Vec<i64>>,
    reviewed_task_ids: Memo<Vec<i64>>,
    is_loaded: Signal<bool>,
    pending_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    reviewed_filter: Signal<Callback<north_stores::TaskModel, bool>>,
    show_reviewed: ReadSignal<bool>,
    set_show_reviewed: WriteSignal<bool>,
    on_task_click: Callback<i64>,
    toolbar: ToolbarConfig,
    show_keybindings_help: RwSignal<bool>,
) -> impl IntoView {
    let review_config = ItemConfig {
        show_review: true,
        ..Default::default()
    };

    view! {
        <div class="space-y-4">
            <PageHeader title="Review" show_keybindings_help=show_keybindings_help />

            <TraversableTaskList
                root_task_ids=review_task_ids
                node_filter=pending_filter
                item_config=review_config
                is_loaded=is_loaded
                allow_create=false
                allow_reorder=false
                on_task_click=on_task_click
                toolbar=toolbar
                show_keybindings_help=show_keybindings_help
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
        </div>
    }
}
