use leptos::prelude::*;
use north_ui::Modal;

use crate::atoms::{Text, TextColor, TextTag, TextVariant};

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[component]
pub fn RecurrenceModalView(
    mode: RwSignal<String>,
    freq: RwSignal<String>,
    interval: RwSignal<String>,
    monday: RwSignal<bool>,
    tuesday: RwSignal<bool>,
    wednesday: RwSignal<bool>,
    thursday: RwSignal<bool>,
    friday: RwSignal<bool>,
    saturday: RwSignal<bool>,
    sunday: RwSignal<bool>,
    time: RwSignal<String>,
    monthday: RwSignal<String>,
    month: RwSignal<String>,
    timezone: Signal<String>,
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

    let is_custom = RwSignal::new({
        let i: u32 = interval.get_untracked().parse().unwrap_or(1);
        i != 1
    });

    let active_preset = Signal::derive(move || {
        if is_custom.get() {
            return "custom";
        }
        match freq.get().as_str() {
            "DAILY" => "daily",
            "WEEKLY" => "weekly",
            "MONTHLY" => "monthly",
            "YEARLY" => "yearly",
            _ => "custom",
        }
    });

    let is_weekly = Signal::derive(move || freq.get() == "WEEKLY");
    let is_monthly = Signal::derive(move || freq.get() == "MONTHLY");
    let is_yearly = Signal::derive(move || freq.get() == "YEARLY");
    let show_monthday = Signal::derive(move || is_monthly.get() || is_yearly.get());

    let select_preset = move |preset: &str| match preset {
        "daily" => {
            freq.set("DAILY".into());
            interval.set("1".into());
            is_custom.set(false);
        }
        "weekly" => {
            freq.set("WEEKLY".into());
            interval.set("1".into());
            is_custom.set(false);
        }
        "monthly" => {
            freq.set("MONTHLY".into());
            interval.set("1".into());
            is_custom.set(false);
        }
        "yearly" => {
            freq.set("YEARLY".into());
            interval.set("1".into());
            is_custom.set(false);
        }
        "custom" => {
            is_custom.set(true);
        }
        _ => {}
    };

    let is_plural = Signal::derive(move || {
        let n: u32 = interval.get().parse().unwrap_or(1);
        n != 1
    });

    let summary = Signal::derive(move || {
        let f = freq.get();
        let i: u32 = interval.get().parse().unwrap_or(1);

        let unit = match f.as_str() {
            "DAILY" => "day",
            "WEEKLY" => "week",
            "MONTHLY" => "month",
            "YEARLY" => "year",
            _ => "day",
        };

        let base = if i == 1 {
            format!("Every {unit}")
        } else {
            format!("Every {i} {unit}s")
        };

        let mut result = base;

        if f == "WEEKLY" {
            let mut days = Vec::new();
            if monday.get() {
                days.push("MO");
            }
            if tuesday.get() {
                days.push("TU");
            }
            if wednesday.get() {
                days.push("WE");
            }
            if thursday.get() {
                days.push("TH");
            }
            if friday.get() {
                days.push("FR");
            }
            if saturday.get() {
                days.push("SA");
            }
            if sunday.get() {
                days.push("SU");
            }
            if !days.is_empty() {
                result = format!("{result} ({days})", days = days.join(","));
            }
        }

        if f == "YEARLY" {
            let mo = month.get();
            let md = monthday.get();
            if let Ok(mo_num) = mo.parse::<usize>() {
                if (1..=12).contains(&mo_num) {
                    let month_name = MONTH_NAMES[mo_num - 1];
                    if !md.is_empty() {
                        result = format!("{result} on {month_name} {md}");
                    } else {
                        result = format!("{result} in {month_name}");
                    }
                }
            } else if !md.is_empty() {
                result = format!("{result} on the {suffix}", suffix = ordinal_suffix(&md));
            }
        } else if f == "MONTHLY" {
            let md = monthday.get();
            if !md.is_empty() {
                result = format!("{result} on the {suffix}", suffix = ordinal_suffix(&md));
            }
        }

        let t = time.get();
        if !t.is_empty() {
            if let Some(formatted) = format_time_12h(&t) {
                result = format!("{result} at {formatted}");
            }
        }

        result
    });

    let show_custom_row = Signal::derive(move || is_custom.get());

    view! {
        <Modal open=open_read set_open=open_write>
            <div class="p-5 space-y-4">
                <Text variant=TextVariant::HeadingSm>"Recurrence"</Text>

                // Preset chips
                <div class="flex flex-wrap gap-1.5">
                    <PresetChip
                        label="Daily"
                        active=Signal::derive(move || active_preset.get() == "daily")
                        on_click=Callback::new(move |()| select_preset("daily"))
                    />
                    <PresetChip
                        label="Weekly"
                        active=Signal::derive(move || active_preset.get() == "weekly")
                        on_click=Callback::new(move |()| select_preset("weekly"))
                    />
                    <PresetChip
                        label="Monthly"
                        active=Signal::derive(move || active_preset.get() == "monthly")
                        on_click=Callback::new(move |()| select_preset("monthly"))
                    />
                    <PresetChip
                        label="Yearly"
                        active=Signal::derive(move || active_preset.get() == "yearly")
                        on_click=Callback::new(move |()| select_preset("yearly"))
                    />
                    <PresetChip
                        label="Custom"
                        active=Signal::derive(move || active_preset.get() == "custom")
                        on_click=Callback::new(move |()| select_preset("custom"))
                    />
                </div>

                // Custom row: Every [N] [unit]
                <Show when=move || show_custom_row.get()>
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
                            prop:value=move || interval.get()
                            on:input=move |ev| {
                                interval.set(event_target_value(&ev));
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
                            prop:value=move || freq.get()
                            on:change=move |ev| {
                                freq.set(event_target_value(&ev));
                            }
                        >
                            <option value="DAILY">
                                {move || if is_plural.get() {
                                    "days"
                                } else {
                                    "day"
                                }}
                            </option>
                            <option value="WEEKLY">
                                {move || if is_plural.get() {
                                    "weeks"
                                } else {
                                    "week"
                                }}
                            </option>
                            <option value="MONTHLY">
                                {move || if is_plural.get() {
                                    "months"
                                } else {
                                    "month"
                                }}
                            </option>
                            <option value="YEARLY">
                                {move || if is_plural.get() {
                                    "years"
                                } else {
                                    "year"
                                }}
                            </option>
                        </select>
                    </div>
                </Show>

                // Day-of-week row (weekly only)
                <Show when=move || is_weekly.get()>
                    <div class="flex gap-1">
                        <DayCheckbox label="Mo" checked=monday/>
                        <DayCheckbox label="Tu" checked=tuesday/>
                        <DayCheckbox label="We" checked=wednesday/>
                        <DayCheckbox label="Th" checked=thursday/>
                        <DayCheckbox label="Fr" checked=friday/>
                        <DayCheckbox label="Sa" checked=saturday/>
                        <DayCheckbox label="Su" checked=sunday/>
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
                            prop:value=move || month.get()
                            on:change=move |ev| {
                                month.set(event_target_value(&ev));
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
                            prop:value=move || monthday.get()
                            on:change=move |ev| {
                                monthday.set(event_target_value(&ev));
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
                            prop:value=move || time.get()
                            on:input=move |ev| {
                                time.set(event_target_value(&ev));
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
                            {move || timezone.get()}
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
                                mode.get() == "scheduled"
                            })
                            on_click=Callback::new(move |()| {
                                mode.set("scheduled".into());
                            })
                        />
                        <RadioChip
                            label="After completion"
                            active=Signal::derive(move || {
                                mode.get() == "after_completion"
                            })
                            on_click=Callback::new(move |()| {
                                mode.set("after_completion".into());
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
fn DayCheckbox(label: &'static str, checked: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            on:click=move |_| checked.update(|v| *v = !*v)
            class=move || {
                if checked.get() {
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

fn ordinal_suffix(day: &str) -> String {
    let n: u32 = day.parse().unwrap_or(0);
    let suffix = match n % 10 {
        1 if n % 100 != 11 => "st",
        2 if n % 100 != 12 => "nd",
        3 if n % 100 != 13 => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn format_time_12h(t: &str) -> Option<String> {
    let mut parts = t.split(':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let (h12, ampm) = if h == 0 {
        (12, "AM")
    } else if h < 12 {
        (h, "AM")
    } else if h == 12 {
        (12, "PM")
    } else {
        (h - 12, "PM")
    };
    if m == 0 {
        Some(format!("{h12} {ampm}"))
    } else {
        Some(format!("{h12}:{m:02} {ampm}"))
    }
}
