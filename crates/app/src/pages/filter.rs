use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::components::task_list::TaskList;
use crate::server_fns::filters::*;
use crate::stores::task_store::TaskStore;

#[component]
pub fn FilterPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    let filter_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
    });

    let (query_text, set_query_text) = signal(String::new());
    let (title_text, set_title_text) = signal("Untitled Filter".to_string());
    let (parse_error, set_parse_error) = signal(Option::<String>::None);
    let (is_dirty, set_is_dirty) = signal(false);

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
            set_is_dirty.set(false);
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

    // Execute filter results
    let filter_results = Resource::new(
        move || query_text.get(),
        |q| async move {
            if q.trim().is_empty() {
                return Ok(vec![]);
            }
            execute_filter(q).await
        },
    );

    let store = TaskStore::new(filter_results);

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
            set_is_dirty.set(false);
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

    let on_save = move |_| {
        let title = title_text.get_untracked();
        let query = query_text.get_untracked();
        if query.trim().is_empty() {
            return;
        }
        if parse_error.get_untracked().is_some() {
            return;
        }
        save_action.dispatch((filter_id.get_untracked(), title, query));
    };

    let on_delete = move |_| {
        if let Some(id) = filter_id.get_untracked() {
            delete_action.dispatch(id);
        }
    };

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <input
                    type="text"
                    class="text-xl font-semibold text-text-primary \
                           bg-transparent border-none outline-none \
                           placeholder:text-text-tertiary w-full"
                    placeholder="Untitled Filter"
                    prop:value=move || title_text.get()
                    on:input=move |ev| {
                        set_title_text.set(event_target_value(&ev));
                        set_is_dirty.set(true);
                    }
                />
                <a
                    href="/filters/help"
                    target="_blank"
                    class="ml-2 px-2 py-1 text-xs text-text-secondary \
                           hover:text-text-primary bg-bg-tertiary \
                           rounded transition-colors flex-shrink-0"
                    title="Query syntax reference"
                >
                    "?"
                </a>
            </div>

            <div>
                <textarea
                    class="w-full bg-bg-input border border-border rounded \
                           px-3 py-2 text-sm text-text-primary \
                           font-mono placeholder:text-text-tertiary \
                           focus:outline-none focus:border-accent \
                           resize-y min-h-20"
                    placeholder="e.g. status = 'ACTIVE' AND tags =~ 'work:*'"
                    rows="3"
                    prop:value=move || query_text.get()
                    on:input=move |ev| {
                        set_query_text.set(event_target_value(&ev));
                        set_is_dirty.set(true);
                    }
                />
                <Show when=move || parse_error.get().is_some()>
                    <p class="text-xs text-red-400 mt-1">
                        {move || parse_error.get().unwrap_or_default()}
                    </p>
                </Show>
            </div>

            <div class="flex gap-2">
                <button
                    class="px-3 py-1.5 text-sm bg-accent text-white \
                           rounded hover:bg-accent-hover transition-colors \
                           disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click=on_save
                    disabled=move || {
                        parse_error.get().is_some()
                            || query_text.get().trim().is_empty()
                    }
                >
                    {move || {
                        if filter_id.get().is_some() {
                            "Update"
                        } else {
                            "Save"
                        }
                    }}
                </button>
                <Show when=move || filter_id.get().is_some()>
                    <button
                        class="px-3 py-1.5 text-sm text-red-400 \
                               hover:text-red-300 transition-colors"
                        on:click=on_delete
                    >
                        "Delete"
                    </button>
                </Show>
                <Show when=move || is_dirty.get()>
                    <span class="text-xs text-text-tertiary self-center">
                        "Unsaved changes"
                    </span>
                </Show>
            </div>

            <hr class="border-border"/>

            <TaskList
                resource=filter_results
                store=store
                empty_message="No matching tasks. Try adjusting your query."
            />
        </div>
    }
}
