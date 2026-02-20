use leptos::prelude::*;

use crate::atoms::{Text, TextColor, TextTag, TextVariant};
use crate::constants::TIMEZONE_GROUPS;

#[component]
pub fn SettingsView(
    interval: ReadSignal<String>,
    set_interval: WriteSignal<String>,
    timezone: ReadSignal<String>,
    set_timezone: WriteSignal<String>,
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
                                set_interval.set(event_target_value(&ev));
                            }
                            class="w-24 bg-bg-input border border-border \
                                   rounded px-3 py-1.5 text-sm \
                                   text-text-primary focus:outline-none \
                                   focus:border-accent"
                        />
                    </div>

                    <div class="space-y-2">
                        <Text variant=TextVariant::LabelLg color=TextColor::Secondary tag=TextTag::Label class="block">
                            "Timezone"
                        </Text>
                        <Text variant=TextVariant::BodySm color=TextColor::Tertiary tag=TextTag::P>
                            "Used for scheduling recurring tasks."
                        </Text>
                        <select
                            on:change=move |ev| {
                                set_timezone.set(event_target_value(&ev));
                            }
                            class="w-64 bg-bg-input border border-border \
                                   rounded px-3 py-1.5 text-sm \
                                   text-text-primary focus:outline-none \
                                   focus:border-accent"
                        >
                            <option value="UTC" selected=move || timezone.get() == "UTC">"UTC"</option>
                            {TIMEZONE_GROUPS
                                .iter()
                                .map(|(label, zones)| {
                                    view! {
                                        <optgroup label=*label>
                                            {zones
                                                .iter()
                                                .map(|tz| {
                                                    let tz = *tz;
                                                    view! {
                                                        <option
                                                            value=tz
                                                            selected=move || timezone.get() == tz
                                                        >
                                                            {tz}
                                                        </option>
                                                    }
                                                })
                                                .collect_view()}
                                        </optgroup>
                                    }
                                })
                                .collect_view()}
                        </select>
                    </div>

                    <button
                        on:click=move |_| on_save.run(())
                        class="px-4 py-1.5 text-sm bg-accent \
                               text-on-accent rounded \
                               hover:bg-accent-hover \
                               transition-colors"
                    >
                        "Save"
                    </button>
                </div>
            </Show>
        </div>
    }
}
