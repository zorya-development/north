use leptos::prelude::*;

use crate::components::icons::{Icon, IconKind};

#[component]
pub fn DateTimePickerView(
    task_id: i64,
    has_start_at: bool,
    start_at_display: Option<String>,
    initial_date: String,
    initial_time: String,
    popover_open: ReadSignal<bool>,
    set_popover_open: WriteSignal<bool>,
    picked_date: RwSignal<String>,
    picked_time: RwSignal<String>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
) -> impl IntoView {
    view! {
        <div class="relative inline-flex">
            {if has_start_at {
                let display = start_at_display.clone().unwrap_or_default();
                let initial_date = initial_date.clone();
                let initial_time = initial_time.clone();
                view! {
                    <button
                        class="inline-flex items-center gap-1 text-accent \
                               hover:bg-bg-tertiary px-1.5 py-0.5 rounded \
                               transition-colors"
                        on:click={
                            let id = initial_date.clone();
                            let it = initial_time.clone();
                            move |_| {
                                picked_date.set(id.clone());
                                picked_time.set(it.clone());
                                set_popover_open.update(|o| *o = !*o);
                            }
                        }
                    >
                        <Icon kind=IconKind::Calendar class="w-3 h-3"/>
                        {display}
                        <span
                            class="hover:text-text-primary ml-0.5 cursor-pointer"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                on_clear_start_at.run(task_id);
                            }
                        >
                            "\u{00d7}"
                        </span>
                    </button>
                }
                .into_any()
            } else {
                view! {
                    <button
                        class="inline-flex items-center gap-1 \
                               text-text-tertiary hover:text-text-secondary \
                               hover:bg-bg-tertiary px-1.5 py-0.5 rounded \
                               transition-colors opacity-0 \
                               group-hover:opacity-100"
                        on:click=move |_| {
                            picked_date.set(String::new());
                            picked_time.set("09:00".to_string());
                            set_popover_open.update(|o| *o = !*o);
                        }
                    >
                        <Icon kind=IconKind::Calendar class="w-3 h-3"/>
                        "Start"
                    </button>
                }
                .into_any()
            }}

            <Show when=move || popover_open.get()>
                <div
                    class="fixed inset-0 z-40"
                    on:click=move |_| set_popover_open.set(false)
                />
                <div class="absolute top-full left-0 mt-1 z-50 \
                            bg-bg-secondary border border-border \
                            rounded-lg shadow-lg p-3 w-[220px]">
                    <div class="flex flex-col gap-2">
                        <label class="text-xs text-text-secondary">"Date"</label>
                        <input
                            type="date"
                            class="bg-bg-input border border-border \
                                   rounded px-2 py-1.5 text-sm \
                                   text-text-primary w-full \
                                   focus:outline-none focus:border-accent"
                            bind:value=picked_date
                            on:change:target=move |ev| {
                                picked_date.set(ev.target().value());
                            }
                        />
                        <label class="text-xs text-text-secondary">"Time"</label>
                        <input
                            type="time"
                            class="bg-bg-input border border-border \
                                   rounded px-2 py-1.5 text-sm \
                                   text-text-primary w-full \
                                   focus:outline-none focus:border-accent"
                            bind:value=picked_time
                            on:change:target=move |ev| {
                                picked_time.set(ev.target().value());
                            }
                        />
                        <div class="flex items-center gap-2 mt-1 pt-2 \
                                    border-t border-border">
                            {has_start_at.then(|| {
                                view! {
                                    <button
                                        class="text-xs text-text-tertiary \
                                               hover:text-accent \
                                               transition-colors"
                                        on:click=move |_| {
                                            set_popover_open.set(false);
                                            on_clear_start_at.run(task_id);
                                        }
                                    >
                                        "Remove"
                                    </button>
                                }
                            })}
                            <div class="flex-1" />
                            <button
                                class="text-xs text-text-secondary \
                                       hover:text-text-primary px-2 py-1 \
                                       rounded transition-colors"
                                on:click=move |_| {
                                    set_popover_open.set(false);
                                }
                            >
                                "Cancel"
                            </button>
                            <button
                                class="text-xs bg-accent \
                                       hover:bg-accent-hover text-white \
                                       px-3 py-1 rounded transition-colors"
                                on:click=move |_| {
                                    let d = picked_date.get_untracked();
                                    let t = picked_time.get_untracked();
                                    if !d.is_empty() {
                                        let time = if t.is_empty() {
                                            "09:00".to_string()
                                        } else {
                                            t
                                        };
                                        let val = format!("{d}T{time}");
                                        set_popover_open.set(false);
                                        on_set_start_at.run((task_id, val));
                                    }
                                }
                            >
                                "Save"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
