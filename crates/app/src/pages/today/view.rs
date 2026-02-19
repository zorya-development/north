use leptos::prelude::*;
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextVariant};
use crate::components::keybindings_modal::KeybindingsModal;
use crate::containers::traversable_task_list::{TraversableTaskList, TtlHandle};

#[component]
pub fn TodayView(
    root_task_ids: Memo<Vec<i64>>,
    show_completed: RwSignal<bool>,
    completed_count: Memo<usize>,
    is_loaded: Signal<bool>,
    on_task_click: Callback<i64>,
) -> impl IntoView {
    let show_keybindings_help = RwSignal::new(false);
    let (help_read, help_write) = show_keybindings_help.split();
    let ttl_handle = RwSignal::new(None::<TtlHandle>);

    view! {
        <div class="space-y-4">
            <div>
                <div class="flex items-center justify-between">
                    <Text variant=TextVariant::HeadingLg>"Today"</Text>
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
                        on:click=move |_| {
                            if let Some(h) = ttl_handle.get_untracked() {
                                h.start_create_top();
                            }
                        }
                        class="text-sm text-text-secondary hover:text-accent \
                               transition-colors cursor-pointer"
                    >
                        "+" " Add task"
                    </button>
                </div>
            </div>

            <TraversableTaskList
                root_task_ids=root_task_ids
                show_completed=show_completed
                show_project=true
                is_loaded=is_loaded
                allow_create=false
                allow_reorder=false
                on_task_click=on_task_click
                show_keybindings_help=show_keybindings_help
                handle=ttl_handle
                empty_message="No tasks scheduled for today."
            />

            <KeybindingsModal open=help_read set_open=help_write />
        </div>
    }
}
