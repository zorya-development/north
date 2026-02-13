use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use north_ui::{Icon, IconKind, Modal};

use crate::components::filter_autocomplete::FilterAutocompleteTextarea;
use crate::components::task_detail_modal::{TaskDetailContext, TaskDetailModal};
use crate::components::task_list::TaskList;
use crate::server_fns::filters::*;
use crate::stores::task_store::TaskStore;

#[component]
pub fn FilterPage() -> impl IntoView {
    let open_task_id = RwSignal::new(None::<i64>);
    provide_context(TaskDetailContext { open_task_id });

    let params = use_params_map();
    let navigate = use_navigate();

    let filter_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
    });

    let (query_text, set_query_text) = signal(String::new());
    let (committed_query, set_committed_query) = signal(String::new());
    let (title_text, set_title_text) = signal("Untitled Filter".to_string());
    let (parse_error, set_parse_error) = signal(Option::<String>::None);
    let (original_title, set_original_title) = signal(String::new());
    let (original_query, set_original_query) = signal(String::new());
    let (is_editing_title, set_is_editing_title) = signal(false);
    let (show_save_modal, set_show_save_modal) = signal(false);
    let (modal_title, set_modal_title) = signal(String::new());
    let modal_input_ref = NodeRef::<leptos::html::Input>::new();

    let is_dirty = Memo::new(move |_| {
        title_text.get() != original_title.get()
            || query_text.get() != original_query.get()
    });

    // Load existing filter if editing
    let saved_filter = Resource::new(
        move || filter_id.get(),
        |id| async move {
            match id {
                Some(id) => get_saved_filter(id).await.ok(),
                None => None,
            }
        },
    );

    // Populate fields when saved filter loads
    Effect::new(move || {
        if let Some(Some(f)) = saved_filter.get() {
            set_title_text.set(f.title.clone());
            set_query_text.set(f.query.clone());
            set_committed_query.set(f.query.clone());
            set_original_title.set(f.title);
            set_original_query.set(f.query);
            set_is_editing_title.set(false);
        }
    });

    // Client-side validation
    Effect::new(move || {
        let q = query_text.get();
        if q.trim().is_empty() {
            set_parse_error.set(None);
            return;
        }
        match north_domain::parse_filter(&q) {
            Ok(_) => set_parse_error.set(None),
            Err(errs) => {
                let msg = errs
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                set_parse_error.set(Some(msg));
            }
        }
    });

    // Auto-focus modal input when opened
    Effect::new(move || {
        if show_save_modal.get() {
            if let Some(input) = modal_input_ref.get() {
                let _ = input.focus();
            }
        }
    });

    // Execute filter results (only on explicit run)
    let filter_results = Resource::new(
        move || committed_query.get(),
        |q| async move {
            if q.trim().is_empty() {
                return Ok(vec![]);
            }
            execute_filter(q).await
        },
    );

    let store = TaskStore::new(filter_results);
    let modal_store = store.clone();

    // Save action
    let save_action = Action::new(move |input: &(Option<i64>, String, String)| {
        let (id, title, query) = input.clone();
        async move {
            match id {
                Some(id) => {
                    update_saved_filter(id, Some(title), Some(query)).await
                }
                None => create_saved_filter(title, query).await,
            }
        }
    });

    let navigate_save = navigate.clone();
    Effect::new(move || {
        if let Some(Ok(filter)) = save_action.value().get() {
            set_title_text.set(filter.title.clone());
            set_original_title.set(filter.title.clone());
            set_original_query.set(filter.query.clone());
            set_is_editing_title.set(false);
            if filter_id.get_untracked().is_none() {
                navigate_save(
                    &format!("/filters/{}", filter.id),
                    Default::default(),
                );
            }
        }
    });

    // Delete action
    let delete_action = Action::new(move |id: &i64| {
        let id = *id;
        delete_saved_filter(id)
    });

    let navigate_delete = navigate;
    Effect::new(move || {
        if let Some(Ok(_)) = delete_action.value().get() {
            navigate_delete("/filters/new", Default::default());
        }
    });

    let run_query = move || {
        let q = query_text.get_untracked();
        if !q.trim().is_empty() && parse_error.get_untracked().is_none() {
            set_committed_query.set(q);
        }
    };

    let on_save = move |_| {
        let query = query_text.get_untracked();
        if query.trim().is_empty() || parse_error.get_untracked().is_some() {
            return;
        }
        if filter_id.get_untracked().is_none() {
            // New filter: open modal to enter title
            set_modal_title.set(String::new());
            set_show_save_modal.set(true);
        } else {
            // Existing filter: save directly
            let title = title_text.get_untracked();
            save_action.dispatch((filter_id.get_untracked(), title, query));
        }
    };

    let on_modal_save = move || {
        let title = modal_title.get_untracked();
        if title.trim().is_empty() {
            return;
        }
        let query = query_text.get_untracked();
        set_show_save_modal.set(false);
        save_action.dispatch((None, title, query));
    };

    let on_delete = move |_| {
        if let Some(id) = filter_id.get_untracked() {
            delete_action.dispatch(id);
        }
    };

    view! {
        <div class="space-y-4">
            // Header
            <div class="flex items-center justify-between gap-2">
                // Left side: title
                <div class="flex items-center gap-2 flex-1 min-w-0">
                    <Show
                        when=move || filter_id.get().is_some()
                        fallback=|| view! {
                            <span class="text-2xl font-semibold tracking-tight text-text-primary">
                                "New Filter"
                            </span>
                        }
                    >
                        <Show
                            when=move || is_editing_title.get()
                            fallback=move || view! {
                                <span class="text-2xl font-semibold tracking-tight \
                                             text-text-primary truncate">
                                    {move || title_text.get()}
                                </span>
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
                        >
                            <input
                                type="text"
                                class="text-2xl font-semibold tracking-tight text-text-primary \
                                       bg-transparent border-none outline-none \
                                       placeholder:text-text-tertiary w-full"
                                placeholder="Filter title"
                                prop:value=move || title_text.get()
                                on:input=move |ev| {
                                    set_title_text
                                        .set(event_target_value(&ev));
                                }
                                on:keydown=move |ev| {
                                    match ev.key().as_str() {
                                        "Enter" => {
                                            ev.prevent_default();
                                            set_is_editing_title.set(false);
                                        }
                                        "Escape" => {
                                            set_title_text.set(
                                                original_title
                                                    .get_untracked(),
                                            );
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
                        on:click=on_save
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
                            class="px-2 py-1 text-sm text-danger \
                                   hover:text-danger-hover transition-colors"
                            on:click=on_delete
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
                            value=query_text
                            set_value=set_query_text
                            placeholder="e.g. status = 'ACTIVE' AND tags =~ 'work:*'"
                            rows=1
                            class="w-full bg-bg-input border border-border \
                                   rounded px-3 py-2 text-sm \
                                   text-text-primary font-mono \
                                   placeholder:text-text-tertiary \
                                   focus:outline-none \
                                   focus:border-accent resize-none"
                            on_submit=Callback::new(move |()| run_query())
                        />
                    </div>
                    <button
                        class="w-24 py-0 text-sm bg-accent \
                               hover:bg-accent-hover text-on-accent \
                               rounded transition-colors \
                               disabled:opacity-50 \
                               disabled:cursor-not-allowed flex-shrink-0"
                        title="Run query (Enter)"
                        on:click=move |_| run_query()
                        disabled=move || {
                            parse_error.get().is_some()
                                || query_text.get().trim().is_empty()
                        }
                    >
                        "Search"
                    </button>
                </div>
                <Show when=move || parse_error.get().is_some()>
                    <p class="text-xs text-danger mt-1">
                        {move || parse_error.get().unwrap_or_default()}
                    </p>
                </Show>
            </div>

            <hr class="border-border"/>

            <TaskList
                resource=filter_results
                store=store
                empty_message="No matching tasks. Try adjusting your query."
            />

            {
                let task_ids = Signal::derive(move || {
                    filter_results
                        .get()
                        .and_then(|r| r.ok())
                        .unwrap_or_default()
                        .iter()
                        .map(|t| t.task.id)
                        .collect::<Vec<_>>()
                });
                view! {
                    <TaskDetailModal task_ids=task_ids task_store=modal_store.clone()/>
                }
            }

            // Save filter modal (new filters only)
            <Modal open=show_save_modal set_open=set_show_save_modal>
                <div class="p-4 space-y-4">
                    <h3 class="text-lg font-semibold text-text-primary">
                        "Save Filter"
                    </h3>
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
                                    on_modal_save();
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
                            on:click=move |_| {
                                set_show_save_modal.set(false)
                            }
                        >
                            "Cancel"
                        </button>
                        <button
                            class="px-3 py-1.5 text-sm bg-accent \
                                   hover:bg-accent-hover text-on-accent \
                                   rounded transition-colors \
                                   disabled:opacity-50 \
                                   disabled:cursor-not-allowed"
                            disabled=move || {
                                modal_title.get().trim().is_empty()
                            }
                            on:click=move |_| on_modal_save()
                        >
                            "Save"
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
