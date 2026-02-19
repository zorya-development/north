use leptos::prelude::*;
use north_dto::{Frequency, RecurrenceType, Weekday};
use north_ui::Modal;

use super::controller::RecurrenceController;
use crate::atoms::{Text, TextColor, TextTag, TextVariant};

#[component]
pub fn RecurrenceModalView(
    ctrl: RecurrenceController,
    on_save: Callback<()>,
    on_remove: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let open = RwSignal::new(true);
    let (open_read, open_write) = open.split();
    Effect::new(move || {
        if !open.get() {
            on_close.run(());
        }
    });

    let active_preset = ctrl.active_preset();
    let freq = ctrl.freq();
    let interval = ctrl.interval();
    let is_weekly = Signal::derive(move || freq.get() == Frequency::Weekly);
    let is_monthly = Signal::derive(move || freq.get() == Frequency::Monthly);
    let is_yearly = Signal::derive(move || freq.get() == Frequency::Yearly);
    let show_monthday = Signal::derive(move || is_monthly.get() || is_yearly.get());
    let is_plural = Signal::derive(move || interval.get() != 1);
    let is_custom = ctrl.is_custom();
    let summary = ctrl.summary();
    let time_str = ctrl.time_str();
    let monthday_str = ctrl.by_month_day_str();
    let month_str = ctrl.by_month_str();

    view! {
        <Modal open=open_read set_open=open_write>
            <div class="p-5 space-y-4">
                <Text variant=TextVariant::HeadingSm>"Recurrence"</Text>

                // Preset chips
                <div class="flex flex-wrap gap-1.5">
                    <PresetChip
                        label="Daily"
                        active=Signal::derive(move || active_preset.get() == "daily")
                        on_click=Callback::new(move |()| ctrl.select_preset("daily"))
                    />
                    <PresetChip
                        label="Weekly"
                        active=Signal::derive(move || active_preset.get() == "weekly")
                        on_click=Callback::new(move |()| ctrl.select_preset("weekly"))
                    />
                    <PresetChip
                        label="Monthly"
                        active=Signal::derive(move || active_preset.get() == "monthly")
                        on_click=Callback::new(move |()| ctrl.select_preset("monthly"))
                    />
                    <PresetChip
                        label="Yearly"
                        active=Signal::derive(move || active_preset.get() == "yearly")
                        on_click=Callback::new(move |()| ctrl.select_preset("yearly"))
                    />
                    <PresetChip
                        label="Custom"
                        active=Signal::derive(move || active_preset.get() == "custom")
                        on_click=Callback::new(move |()| ctrl.select_preset("custom"))
                    />
                </div>

                // Custom row: Every [N] [unit]
                <Show when=move || is_custom.get()>
                    <div class="flex items-center gap-2">
                        <Text
                            variant=TextVariant::LabelLg
                            color=TextColor::Secondary
                        >
                            "Every"
                        </Text>
                        <input
                            type="number"
                            min="1"
                            prop:value=move || interval.get().to_string()
                            on:input=move |ev| {
                                if let Ok(n) = event_target_value(&ev).parse::<u32>() {
                                    ctrl.set_interval(n);
                                }
                            }
                            class="w-16 bg-bg-input border border-border \
                                   rounded px-2 py-1.5 text-sm \
                                   text-text-primary text-center \
                                   focus:outline-none \
                                   focus:border-accent"
                        />
                        <select
                            class="bg-bg-input border border-border \
                                   rounded px-3 py-1.5 text-sm \
                                   text-text-primary focus:outline-none \
                                   focus:border-accent"
                            prop:value=move || freq.get().code()
                            on:change=move |ev| {
                                ctrl.set_freq_from_code(&event_target_value(&ev));
                            }
                        >
                            <option value="DAILY">
                                {move || if is_plural.get() { "days" } else { "day" }}
                            </option>
                            <option value="WEEKLY">
                                {move || if is_plural.get() { "weeks" } else { "week" }}
                            </option>
                            <option value="MONTHLY">
                                {move || if is_plural.get() { "months" } else { "month" }}
                            </option>
                            <option value="YEARLY">
                                {move || if is_plural.get() { "years" } else { "year" }}
                            </option>
                        </select>
                    </div>
                </Show>

                // Day-of-week row (weekly only)
                <Show when=move || is_weekly.get()>
                    <div class="flex gap-1">
                        {Weekday::ALL
                            .into_iter()
                            .map(|day| {
                                let selected = ctrl.is_day_selected(day);
                                view! {
                                    <DayCheckbox
                                        label=day.label()
                                        selected=selected
                                        on_toggle=Callback::new(move |()| {
                                            ctrl.toggle_day(day);
                                        })
                                    />
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>
                </Show>

                // Month select (yearly only)
                <Show when=move || is_yearly.get()>
                    <div class="space-y-1.5">
                        <Text
                            variant=TextVariant::LabelLg
                            color=TextColor::Secondary
                            tag=TextTag::Label
                            class="block"
                        >
                            "Month"
                        </Text>
                        <select
                            class="w-full bg-bg-input border border-border \
                                   rounded px-3 py-1.5 text-sm \
                                   text-text-primary focus:outline-none \
                                   focus:border-accent"
                            prop:value=move || month_str.get()
                            on:change=move |ev| {
                                ctrl.set_month(&event_target_value(&ev));
                            }
                        >
                            <option value="">"—"</option>
                            <option value="1">"January"</option>
                            <option value="2">"February"</option>
                            <option value="3">"March"</option>
                            <option value="4">"April"</option>
                            <option value="5">"May"</option>
                            <option value="6">"June"</option>
                            <option value="7">"July"</option>
                            <option value="8">"August"</option>
                            <option value="9">"September"</option>
                            <option value="10">"October"</option>
                            <option value="11">"November"</option>
                            <option value="12">"December"</option>
                        </select>
                    </div>
                </Show>

                // Day-of-month select (monthly + yearly)
                <Show when=move || show_monthday.get()>
                    <div class="space-y-1.5">
                        <Text
                            variant=TextVariant::LabelLg
                            color=TextColor::Secondary
                            tag=TextTag::Label
                            class="block"
                        >
                            "Day of month"
                        </Text>
                        <select
                            class="w-full bg-bg-input border border-border \
                                   rounded px-3 py-1.5 text-sm \
                                   text-text-primary focus:outline-none \
                                   focus:border-accent"
                            prop:value=move || monthday_str.get()
                            on:change=move |ev| {
                                ctrl.set_month_day(&event_target_value(&ev));
                            }
                        >
                            <option value="">"—"</option>
                            {(1..=31)
                                .map(|d| {
                                    let val = d.to_string();
                                    view! {
                                        <option value=val.clone()>
                                            {val.clone()}
                                        </option>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </div>
                </Show>

                // Time input (all frequencies)
                <div class="space-y-1.5">
                    <Text
                        variant=TextVariant::LabelLg
                        color=TextColor::Secondary
                        tag=TextTag::Label
                        class="block"
                    >
                        "Time"
                    </Text>
                    <div class="flex items-center gap-2">
                        <input
                            type="time"
                            prop:value=move || time_str.get()
                            on:input=move |ev| {
                                ctrl.set_time(&event_target_value(&ev));
                            }
                            class="bg-bg-input border border-border \
                                   rounded px-3 py-1.5 text-sm \
                                   text-text-primary focus:outline-none \
                                   focus:border-accent"
                        />
                        <Text
                            variant=TextVariant::BodySm
                            color=TextColor::Tertiary
                        >
                            {move || ctrl.timezone.get()}
                        </Text>
                    </div>
                </div>

                // Timing toggle
                <div class="space-y-1.5">
                    <Text
                        variant=TextVariant::LabelLg
                        color=TextColor::Secondary
                        tag=TextTag::Label
                        class="block"
                    >
                        "Timing"
                    </Text>
                    <div class="flex gap-1.5">
                        <RadioChip
                            label="From due date"
                            active=Signal::derive(move || {
                                ctrl.recurrence_type.get()
                                    == RecurrenceType::Scheduled
                            })
                            on_click=Callback::new(move |()| {
                                ctrl.set_recurrence_type(
                                    RecurrenceType::Scheduled,
                                );
                            })
                        />
                        <RadioChip
                            label="After completion"
                            active=Signal::derive(move || {
                                ctrl.recurrence_type.get()
                                    == RecurrenceType::AfterCompletion
                            })
                            on_click=Callback::new(move |()| {
                                ctrl.set_recurrence_type(
                                    RecurrenceType::AfterCompletion,
                                );
                            })
                        />
                    </div>
                </div>

                // Summary
                <Text
                    variant=TextVariant::BodySm
                    color=TextColor::Tertiary
                    tag=TextTag::P
                >
                    {summary}
                </Text>

                // Footer
                <div
                    class="flex items-center justify-between pt-2 \
                           border-t border-border"
                >
                    <button
                        on:click=move |_| on_remove.run(())
                        class="text-sm text-danger hover:text-danger/80 \
                               transition-colors"
                    >
                        "Remove"
                    </button>
                    <div class="flex gap-2">
                        <button
                            on:click=move |_| on_close.run(())
                            class="px-3 py-1.5 text-sm text-text-secondary \
                                   hover:text-text-primary transition-colors"
                        >
                            "Cancel"
                        </button>
                        <button
                            on:click=move |_| on_save.run(())
                            class="px-4 py-1.5 text-sm bg-accent \
                                   text-on-accent rounded \
                                   hover:bg-accent-hover transition-colors"
                        >
                            "Save"
                        </button>
                    </div>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn PresetChip(label: &'static str, active: Signal<bool>, on_click: Callback<()>) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click.run(())
            class=move || {
                if active.get() {
                    "px-3 py-1.5 rounded-full text-sm font-medium \
                     bg-accent text-on-accent transition-colors"
                } else {
                    "px-3 py-1.5 rounded-full text-sm font-medium \
                     bg-bg-input text-text-secondary \
                     border border-border \
                     hover:border-accent hover:text-text-primary \
                     transition-colors"
                }
            }
        >
            {label}
        </button>
    }
}

#[component]
fn RadioChip(label: &'static str, active: Signal<bool>, on_click: Callback<()>) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click.run(())
            class=move || {
                if active.get() {
                    "px-3 py-1.5 rounded-full text-sm font-medium \
                     bg-accent text-on-accent transition-colors"
                } else {
                    "px-3 py-1.5 rounded-full text-sm font-medium \
                     bg-bg-input text-text-secondary \
                     border border-border \
                     hover:border-accent hover:text-text-primary \
                     transition-colors"
                }
            }
        >
            {label}
        </button>
    }
}

#[component]
fn DayCheckbox(
    label: &'static str,
    selected: Signal<bool>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_toggle.run(())
            class=move || {
                if selected.get() {
                    "w-8 h-8 rounded-md text-xs font-medium \
                     bg-accent text-on-accent transition-colors"
                } else {
                    "w-8 h-8 rounded-md text-xs font-medium \
                     bg-bg-input text-text-tertiary \
                     border border-border \
                     hover:border-accent hover:text-text-secondary \
                     transition-colors"
                }
            }
        >
            {label}
        </button>
    }
}
