use leptos::prelude::*;

use crate::server_fns::settings::{get_user_settings, update_review_interval};

#[component]
pub fn SettingsPage() -> impl IntoView {
    let settings = Resource::new(|| (), |_| get_user_settings());

    let (interval, set_interval) = signal(String::new());
    let (saved, set_saved) = signal(false);

    let save_action = Action::new(|days: &i16| {
        let days = *days;
        update_review_interval(days)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = save_action.value().get() {
            set_saved.set(true);
        }
    });

    view! {
        <div class="space-y-6 max-w-lg">
            <h1 class="text-xl font-semibold text-text-primary">"Settings"</h1>

            <Suspense fallback=move || {
                view! {
                    <div class="text-sm text-text-secondary py-4">
                        "Loading settings..."
                    </div>
                }
            }>
                {move || {
                    Suspend::new(async move {
                        match settings.await {
                            Ok(s) => {
                                if interval.get_untracked().is_empty() {
                                    set_interval
                                        .set(s.review_interval_days.to_string());
                                }
                                view! {
                                    <div class="space-y-4">
                                        <div class="space-y-2">
                                            <label class="block text-sm \
                                                          font-medium \
                                                          text-text-secondary">
                                                "Review interval (days)"
                                            </label>
                                            <p class="text-xs text-text-tertiary">
                                                "Tasks will appear in Review \
                                                 after this many days since \
                                                 their last review."
                                            </p>
                                            <input
                                                type="number"
                                                min="1"
                                                prop:value=move || interval.get()
                                                on:input=move |ev| {
                                                    set_saved.set(false);
                                                    set_interval
                                                        .set(event_target_value(&ev));
                                                }
                                                class="w-24 bg-bg-input \
                                                       border border-border \
                                                       rounded px-3 py-1.5 \
                                                       text-sm text-text-primary \
                                                       focus:outline-none \
                                                       focus:border-accent"
                                            />
                                        </div>

                                        <div class="flex items-center gap-3">
                                            <button
                                                on:click=move |_| {
                                                    if let Ok(days) = interval
                                                        .get_untracked()
                                                        .parse::<i16>()
                                                    {
                                                        if days >= 1 {
                                                            save_action
                                                                .dispatch(days);
                                                        }
                                                    }
                                                }
                                                class="px-4 py-1.5 text-sm \
                                                       bg-accent text-white \
                                                       rounded \
                                                       hover:bg-accent-hover \
                                                       transition-colors"
                                            >
                                                "Save"
                                            </button>
                                            <Show when=move || saved.get()>
                                                <span class="text-sm \
                                                             text-green-500">
                                                    "Saved"
                                                </span>
                                            </Show>
                                        </div>
                                    </div>
                                }
                                .into_any()
                            }
                            Err(e) => {
                                view! {
                                    <div class="text-sm text-red-500 py-4">
                                        {format!("Failed to load settings: {e}")}
                                    </div>
                                }
                                .into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
