use leptos::prelude::*;

use crate::atoms::{Text, TextColor, TextTag, TextVariant};

#[component]
pub fn SettingsView(
    interval: ReadSignal<String>,
    set_interval: WriteSignal<String>,
    saved: ReadSignal<bool>,
    set_saved: WriteSignal<bool>,
    is_loaded: Signal<bool>,
    on_save: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="space-y-6 max-w-lg">
            <Text variant=TextVariant::HeadingLg>"Settings"</Text>

            <Show
                when=move || is_loaded.get()
                fallback=|| {
                    view! {
                        <Text variant=TextVariant::BodyMd color=TextColor::Secondary tag=TextTag::P class="py-4">
                            "Loading settings..."
                        </Text>
                    }
                }
            >
                <div class="space-y-4">
                    <div class="space-y-2">
                        <Text variant=TextVariant::LabelLg color=TextColor::Secondary tag=TextTag::Label class="block">
                            "Review interval (days)"
                        </Text>
                        <Text variant=TextVariant::BodySm color=TextColor::Tertiary tag=TextTag::P>
                            "Tasks will appear in Review after this many \
                             days since their last review."
                        </Text>
                        <input
                            type="number"
                            min="1"
                            prop:value=move || interval.get()
                            on:input=move |ev| {
                                set_saved.set(false);
                                set_interval.set(event_target_value(&ev));
                            }
                            class="w-24 bg-bg-input border border-border \
                                   rounded px-3 py-1.5 text-sm \
                                   text-text-primary focus:outline-none \
                                   focus:border-accent"
                        />
                    </div>

                    <div class="flex items-center gap-3">
                        <button
                            on:click=move |_| on_save.run(())
                            class="px-4 py-1.5 text-sm bg-accent \
                                   text-on-accent rounded \
                                   hover:bg-accent-hover \
                                   transition-colors"
                        >
                            "Save"
                        </button>
                        <Show when=move || saved.get()>
                            <span class="text-sm text-success">"Saved"</span>
                        </Show>
                    </div>
                </div>
            </Show>
        </div>
    }
}
