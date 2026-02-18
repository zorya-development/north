use leptos::prelude::*;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant};
use crate::components::keybindings_modal::KeybindingsModal;
use crate::containers::task_list::TaskList;
use crate::containers::traversable_task_list::TraversableTaskList;

#[component]
pub fn ReviewView(
    review_task_ids: Memo<Vec<i64>>,
    reviewed_task_ids: Memo<Vec<i64>>,
    is_loaded: Signal<bool>,
    show_reviewed: ReadSignal<bool>,
    set_show_reviewed: WriteSignal<bool>,
    on_review_all: Callback<()>,
    on_task_click: Callback<i64>,
) -> impl IntoView {
    let empty_reorder_tasks = Memo::new(|_| vec![]);

    let show_keybindings_help = RwSignal::new(false);
    let (help_read, help_write) = show_keybindings_help.split();

    // Review tasks are always active, show_completed is always true
    // so the traversable list shows all of them.
    let show_completed = RwSignal::new(true);

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <Text variant=TextVariant::HeadingMd>"Review"</Text>
                <div class="flex items-center gap-2">
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
                    <button
                        on:click=move |_| on_review_all.run(())
                        class="px-3 py-1.5 text-sm bg-accent text-white rounded \
                               hover:bg-accent-hover transition-colors"
                    >
                        "Mark All as Reviewed"
                    </button>
                </div>
            </div>

            <TraversableTaskList
                root_task_ids=review_task_ids
                show_completed=show_completed
                show_project=true
                show_review=true
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
                        <TaskList
                            active_task_ids=reviewed_task_ids
                            active_tasks_for_reorder=empty_reorder_tasks
                            is_loaded=is_loaded
                            show_review=true
                            show_project=true
                            on_reorder=Callback::new(|_| {})
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
