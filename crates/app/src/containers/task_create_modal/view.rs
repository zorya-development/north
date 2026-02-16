use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use north_dto::CreateTask;
use north_stores::{AppStore, TaskCreateModalStore};
use north_ui::{Icon, IconKind};

use crate::atoms::{Text, TextColor, TextVariant};
use crate::components::date_picker::DateTimePicker;
use crate::containers::autocomplete::{AutocompleteInput, AutocompleteTextarea};
use crate::containers::project_picker::ProjectPicker;
use crate::containers::tag_picker::TagPicker;

#[component]
pub fn TaskCreateModalView(store: TaskCreateModalStore) -> impl IntoView {
    let app_store = expect_context::<AppStore>();

    let (title, set_title) = signal(String::new());
    let (body, set_body) = signal(String::new());
    let (project_id, set_project_id) = signal(store.default_project_id());
    let (parent_id, _set_parent_id) = signal(store.default_parent_id());
    let (start_at, set_start_at) = signal(None::<String>);
    let (_tag_names, set_tag_names) = signal(Vec::<String>::new());

    let project_title = Memo::new(move |_| {
        project_id.get().and_then(|pid| {
            app_store
                .projects
                .get()
                .into_iter()
                .find(|p| p.id == pid)
                .map(|p| p.title.clone())
        })
    });

    let submit = Callback::new(move |()| {
        let t = title.get_untracked().trim().to_string();
        if t.is_empty() {
            return;
        }
        let b = body.get_untracked().trim().to_string();
        let body_opt = if b.is_empty() { None } else { Some(b) };

        let sa = start_at.get_untracked().and_then(|s| {
            chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S"))
                .ok()
                .map(|dt| dt.and_utc())
        });

        store.create_task(CreateTask {
            title: t,
            body: body_opt,
            project_id: project_id.get_untracked(),
            parent_id: parent_id.get_untracked(),
            start_at: sa,
            due_date: None,
        });
    });

    let on_form_keydown: std::sync::Arc<dyn Fn(KeyboardEvent) + Send + Sync> =
        std::sync::Arc::new(move |ev: KeyboardEvent| {
            if ev.key() == "Enter" && ev.shift_key() {
                ev.prevent_default();
                submit.run(());
            } else if ev.key() == "Escape" {
                store.close();
            }
        });

    view! {
        <div
            class="fixed inset-0 z-50 flex items-start justify-center pt-[15vh]"
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    store.close();
                }
            }
        >
            // Backdrop
            <div
                class="absolute inset-0 bg-black/50"
                on:click=move |_| store.close()
            />
            // Modal
            <div
                class="relative border border-(--border-muted) \
                       rounded-2xl shadow-2xl max-w-xl w-full mx-4 \
                       flex flex-col"
                style="background-color: var(--bg-secondary)"
                on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
            >
                <div class="p-4 space-y-3">
                    // Title input
                    <AutocompleteInput
                        value=title
                        set_value=set_title
                        placeholder="Task name"
                        class="w-full text-base font-semibold bg-transparent \
                               outline-none no-focus-ring \
                               placeholder:text-text-secondary \
                               text-text-primary"
                        on_keydown=on_form_keydown.clone()
                        autofocus=true
                    />
                    // Description
                    <AutocompleteTextarea
                        value=body
                        set_value=set_body
                        placeholder="Description"
                        rows=2
                        class="w-full text-sm bg-transparent \
                               outline-none no-focus-ring \
                               placeholder:text-text-tertiary \
                               text-text-secondary resize-none"
                        on_keydown=on_form_keydown
                    />
                    // Quick action buttons
                    <div
                        class="flex items-center gap-1"
                        on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                    >
                        <DateTimePicker
                            task_id=0
                            start_at=start_at.get_untracked().and_then(|s| {
                                chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M")
                                    .ok()
                                    .map(|dt| dt.and_utc())
                            })
                            on_set_start_at=Callback::new(move |(_, sa): (i64, String)| {
                                set_start_at.set(Some(sa));
                            })
                            on_clear_start_at=Callback::new(move |_| {
                                set_start_at.set(None);
                            })
                            icon_only=true
                        />
                        <ProjectPicker
                            task_id=0
                            project_id=project_id.get_untracked()
                            project_title=project_title.get_untracked()
                            on_set_project=Callback::new(move |(_, pid): (i64, i64)| {
                                set_project_id.set(Some(pid));
                            })
                            on_clear_project=Callback::new(move |_| {
                                set_project_id.set(None);
                            })
                            icon_only=true
                        />
                        <TagPicker
                            task_id=0
                            tags=vec![]
                            on_set_tags=Callback::new(move |(_, names): (i64, Vec<String>)| {
                                set_tag_names.set(names);
                            })
                            icon_only=true
                        />
                    </div>
                </div>
                // Footer
                <div class="border-t border-(--border-muted) px-4 py-3 flex items-center justify-between">
                    <div class="flex items-center gap-1.5 text-sm text-text-secondary">
                        <Icon kind=IconKind::Folder class="w-4 h-4"/>
                        <Text variant=TextVariant::BodySm color=TextColor::Secondary>
                            {move || {
                                project_title
                                    .get()
                                    .unwrap_or_else(|| "Inbox".to_string())
                            }}
                        </Text>
                    </div>
                    <div class="flex items-center gap-2">
                        <button
                            on:click=move |_| store.close()
                            class="px-3 py-1.5 text-sm text-text-secondary \
                                   hover:text-text-primary transition-colors \
                                   rounded"
                        >
                            "Cancel"
                        </button>
                        <button
                            on:click=move |_| submit.run(())
                            class="px-3 py-1.5 text-sm bg-accent \
                                   text-on-accent rounded-lg \
                                   hover:bg-accent-hover transition-colors \
                                   font-medium"
                        >
                            "Add task"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
