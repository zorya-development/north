use leptos::prelude::*;
use north_stores::use_app_store;
use north_ui::{Icon, IconKind, Modal};

use super::controller::FilterController;
use crate::atoms::{Text, TextColor, TextTag, TextVariant};
use crate::components::filter_autocomplete::FilterAutocompleteTextarea;
use crate::containers::task_list::TaskList;

#[component]
pub fn FilterView(
    ctrl: FilterController,
    on_run_query: Callback<()>,
    on_save: Callback<()>,
    on_save_new: Callback<()>,
    on_delete: Callback<()>,
    on_task_click: Callback<i64>,
) -> impl IntoView {
    let app_store = use_app_store();
    let filter_dsl = app_store.filter_dsl;

    let (title_text, _set_title_text) = ctrl.title_text;
    let (is_editing_title, set_is_editing_title) = ctrl.is_editing_title;
    let (show_save_modal, set_show_save_modal) = ctrl.show_save_modal;
    let (modal_title, set_modal_title) = ctrl.modal_title;
    let (original_title, _) = ctrl.original_title;
    let is_dirty = ctrl.is_dirty;
    let filter_id = ctrl.filter_id;

    let query_text = filter_dsl.query();
    let parse_error = filter_dsl.parse_error();
    let filter_result_ids = Memo::new(move |_| filter_dsl.result_ids().get());
    let is_loaded = filter_dsl.is_loaded();

    let modal_input_ref = NodeRef::<leptos::html::Input>::new();

    // Auto-focus modal input when opened
    Effect::new(move || {
        if show_save_modal.get() {
            if let Some(input) = modal_input_ref.get() {
                let _ = input.focus();
            }
        }
    });

    let empty_reorder_tasks = Memo::new(|_| vec![]);

    view! {
        <div class="space-y-4">
            // Header
            <div class="flex items-center justify-between gap-2">
                // Left side: title
                <div class="flex items-center gap-2 flex-1 min-w-0">
                    <Show
                        when=move || filter_id.get().is_some()
                        fallback=|| {
                            view! {
                                <Text variant=TextVariant::HeadingMd tag=TextTag::Span>
                                    "New Filter"
                                </Text>
                            }
                        }
                    >
                        <Show
                            when=move || is_editing_title.get()
                            fallback=move || {
                                view! {
                                    <Text variant=TextVariant::HeadingMd truncate=true>
                                        {move || title_text.get()}
                                    </Text>
                                    <button
                                        class="p-1 text-text-tertiary \
                                               hover:text-text-primary \
                                               transition-colors flex-shrink-0"
                                        title="Rename filter"
                                        on:click=move |_| {
                                            set_is_editing_title.set(true)
                                        }
                                    >
                                        <Icon kind=IconKind::Edit class="w-4 h-4"/>
                                    </button>
                                }
                            }
                        >
                            <input
                                type="text"
                                class="text-xl font-semibold text-text-primary \
                                       bg-transparent border-none outline-none \
                                       placeholder:text-text-tertiary w-full"
                                placeholder="Filter title"
                                prop:value=move || title_text.get()
                                on:input=move |ev| {
                                    _set_title_text.set(event_target_value(&ev));
                                }
                                on:keydown=move |ev| {
                                    match ev.key().as_str() {
                                        "Enter" => {
                                            ev.prevent_default();
                                            set_is_editing_title.set(false);
                                        }
                                        "Escape" => {
                                            _set_title_text
                                                .set(original_title.get_untracked());
                                            set_is_editing_title.set(false);
                                        }
                                        _ => {}
                                    }
                                }
                                on:blur=move |_| {
                                    set_is_editing_title.set(false);
                                }
                            />
                        </Show>
                    </Show>
                </div>

                // Right side: action icons
                <div class="flex items-center gap-1 flex-shrink-0">
                    // Save icon
                    <button
                        class=move || {
                            let base = "p-1.5 rounded transition-colors";
                            let disabled = parse_error.get().is_some()
                                || query_text.get().trim().is_empty();
                            if disabled {
                                format!(
                                    "{base} text-text-tertiary opacity-50 \
                                     cursor-not-allowed"
                                )
                            } else if is_dirty.get() {
                                format!(
                                    "{base} text-accent hover:text-accent-hover"
                                )
                            } else {
                                format!(
                                    "{base} text-text-tertiary \
                                     hover:text-text-secondary"
                                )
                            }
                        }
                        title=move || {
                            if filter_id.get().is_none() {
                                "Save filter"
                            } else if is_dirty.get() {
                                "Save changes"
                            } else {
                                "No changes to save"
                            }
                        }
                        on:click=move |_| on_save.run(())
                        disabled=move || {
                            parse_error.get().is_some()
                                || query_text.get().trim().is_empty()
                        }
                    >
                        <Icon kind=IconKind::Save class="w-5 h-5"/>
                    </button>

                    // Help icon
                    <a
                        href="/filters/help"
                        target="_blank"
                        class="p-1.5 text-text-tertiary hover:text-text-primary \
                               rounded transition-colors"
                        title="Query syntax reference"
                    >
                        <Icon kind=IconKind::QuestionMark class="w-5 h-5"/>
                    </a>

                    // Delete button (saved filters only)
                    <Show when=move || filter_id.get().is_some()>
                        <button
                            class="px-2 py-1 text-sm text-red-400 \
                                   hover:text-red-300 transition-colors"
                            on:click=move |_| on_delete.run(())
                        >
                            "Delete"
                        </button>
                    </Show>
                </div>
            </div>

            // Search bar
            <div>
                <div class="flex w-full gap-2">
                    <div class="flex-1 min-w-0">
                        <FilterAutocompleteTextarea
                            placeholder="e.g. status = 'ACTIVE' AND tags =~ 'work:*'"
                            rows=1
                            class="w-full bg-bg-input border border-border \
                                   rounded px-3 py-2 text-sm \
                                   text-text-primary font-mono \
                                   placeholder:text-text-tertiary \
                                   focus:outline-none \
                                   focus:border-accent resize-none"
                            on_submit=Callback::new(move |()| on_run_query.run(()))
                        />
                    </div>
                    <button
                        class="w-24 py-0 text-sm bg-accent \
                               hover:bg-accent-hover text-white \
                               rounded transition-colors \
                               disabled:opacity-50 \
                               disabled:cursor-not-allowed flex-shrink-0"
                        title="Run query (Enter)"
                        on:click=move |_| on_run_query.run(())
                        disabled=move || {
                            parse_error.get().is_some()
                                || query_text.get().trim().is_empty()
                        }
                    >
                        "Search"
                    </button>
                </div>
                <Show when=move || parse_error.get().is_some()>
                    <Text variant=TextVariant::BodySm color=TextColor::Danger tag=TextTag::P class="mt-1">
                        {move || parse_error.get().unwrap_or_default()}
                    </Text>
                </Show>
            </div>

            <hr class="border-border"/>

            <TaskList
                active_task_ids=filter_result_ids
                active_tasks_for_reorder=empty_reorder_tasks
                is_loaded=is_loaded
                show_project=true
                on_reorder=Callback::new(|_| {})
                on_task_click=on_task_click
                empty_message="No matching tasks. Try adjusting your query."
            />

            // Save filter modal (new filters only)
            <Modal open=show_save_modal set_open=set_show_save_modal>
                <div class="p-4 space-y-4">
                    <Text variant=TextVariant::HeadingSm>
                        "Save Filter"
                    </Text>
                    <input
                        type="text"
                        node_ref=modal_input_ref
                        class="w-full bg-bg-input border border-border \
                               rounded px-3 py-2 text-sm text-text-primary \
                               placeholder:text-text-tertiary \
                               focus:outline-none focus:border-accent"
                        placeholder="Filter title"
                        prop:value=move || modal_title.get()
                        on:input=move |ev| {
                            set_modal_title.set(event_target_value(&ev));
                        }
                        on:keydown=move |ev| {
                            match ev.key().as_str() {
                                "Enter" => {
                                    ev.prevent_default();
                                    on_save_new.run(());
                                }
                                "Escape" => {
                                    set_show_save_modal.set(false);
                                }
                                _ => {}
                            }
                        }
                    />
                    <div class="flex justify-end gap-2">
                        <button
                            class="px-3 py-1.5 text-sm text-text-secondary \
                                   hover:text-text-primary transition-colors"
                            on:click=move |_| set_show_save_modal.set(false)
                        >
                            "Cancel"
                        </button>
                        <button
                            class="px-3 py-1.5 text-sm bg-accent \
                                   hover:bg-accent-hover text-white \
                                   rounded transition-colors \
                                   disabled:opacity-50 \
                                   disabled:cursor-not-allowed"
                            disabled=move || modal_title.get().trim().is_empty()
                            on:click=move |_| on_save_new.run(())
                        >
                            "Save"
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
