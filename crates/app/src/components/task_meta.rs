use leptos::prelude::*;
use north_dto::{RecurrenceType, TagInfo};
use north_ui::{Icon, IconKind};

#[component]
pub fn TaskMeta(
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    due_date: Option<chrono::NaiveDate>,
    tags: Vec<TagInfo>,
    #[prop(default = None)] reviewed_at: Option<chrono::NaiveDate>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = 0)] subtask_count: i64,
    #[prop(default = 0)] completed_subtask_count: i64,
    #[prop(optional)] on_toggle_subtasks: Option<Callback<()>>,
    #[prop(default = Callback::new(|_| {}))] on_review: Callback<()>,
    #[prop(default = None)] recurrence_type: Option<RecurrenceType>,
    #[prop(default = None)] recurrence_rule: Option<String>,
    #[prop(optional)] on_recurrence_click: Option<Callback<()>>,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let has_recurrence = recurrence_type.is_some();
    let has_meta = start_at.is_some()
        || due_date.is_some()
        || !tags.is_empty()
        || show_review
        || subtask_count > 0
        || has_recurrence;

    has_meta.then(|| {
        let recurrence_label = recurrence_rule
            .as_deref()
            .map(summarize_rrule)
            .unwrap_or_default();
        view! {
            <div class=format!(
                "flex items-center gap-2 text-xs text-text-tertiary flex-wrap {class}"
            )>
                {has_recurrence.then(|| {
                    view! {
                        <span
                            class="inline-flex items-center gap-1 \
                                   text-accent cursor-pointer \
                                   hover:text-accent-hover \
                                   transition-colors"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                if let Some(cb) = on_recurrence_click {
                                    cb.run(());
                                }
                            }
                        >
                            <Icon
                                kind=IconKind::Recurrence
                                class="w-3 h-3"
                            />
                            {recurrence_label.clone()}
                        </span>
                    }
                })}
                {start_at.map(|dt| {
                    let is_overdue = dt < chrono::Utc::now();
                    let class = if is_overdue {
                        "inline-flex items-center gap-1 text-danger"
                    } else {
                        "inline-flex items-center gap-1 text-text-secondary"
                    };
                    let display = dt.format("%b %-d, %-I:%M %p").to_string();
                    view! {
                        <span class=class>
                            <Icon
                                kind=IconKind::Calendar
                                class="w-3 h-3"
                            />
                            {display}
                        </span>
                    }
                })}
                {(subtask_count > 0).then(|| {
                    view! {
                        <span class="inline-flex items-center gap-0.5 \
                                     text-text-secondary \
                                     hover:text-accent cursor-pointer \
                                     transition-colors pl-2"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                if let Some(cb) = on_toggle_subtasks {
                                    cb.run(());
                                }
                            }
                        >
                            <Icon
                                kind=IconKind::Subtask
                                class="w-3 h-3"
                            />
                            {format!(
                                "{completed_subtask_count}/{subtask_count} subtask{}",
                                if subtask_count == 1 { "" } else { "s" },
                            )}
                        </span>
                    }
                })}
                {due_date.map(|d| {
                    let is_overdue = d < chrono::Utc::now().date_naive();
                    let class = if is_overdue {
                        "text-danger"
                    } else {
                        "text-text-secondary"
                    };
                    view! {
                        <span class=class>{format!("Due {d}")}</span>
                    }
                })}
                {(!tags.is_empty()).then(|| {
                    view! {
                        <div class="flex items-center gap-1 flex-wrap">
                            {tags
                                .into_iter()
                                .map(|tag| {
                                    view! {
                                        <span
                                            class="inline-flex items-center \
                                                   gap-0.5 text-xs"
                                            style=format!(
                                                "color: {}",
                                                tag.color,
                                            )
                                        >
                                            <Icon
                                                kind=IconKind::Tag
                                                class="w-3 h-3"
                                            />
                                            {tag.name}
                                        </span>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }
                })}
                {if show_review {
                    Some(view! {
                        <span class="inline-flex items-center gap-1">
                            <button
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    on_review.run(());
                                }
                                class="text-text-secondary \
                                       hover:text-accent \
                                       cursor-pointer \
                                       transition-colors"
                            >
                                "Mark reviewed"
                            </button>
                            <span class="text-text-tertiary">
                                "\u{00b7}"
                            </span>
                            {match reviewed_at {
                                Some(d) => format!("Reviewed {d}"),
                                None => "Never reviewed".to_string(),
                            }}
                        </span>
                    })
                } else {
                    None
                }}
            </div>
        }
    })
}

pub fn summarize_rrule(rule: &str) -> String {
    const MONTH_NAMES: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let mut freq = "";
    let mut interval: u32 = 1;
    let mut byday = String::new();
    let mut byhour = String::new();
    let mut byminute = String::new();
    let mut bymonthday = String::new();
    let mut bymonth = String::new();

    for part in rule.split(';') {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("").trim();
        let val = kv.next().unwrap_or("").trim();
        match key {
            "FREQ" => {
                freq = match val {
                    "DAILY" => "day",
                    "WEEKLY" => "week",
                    "MONTHLY" => "month",
                    "YEARLY" => "year",
                    _ => val,
                }
            }
            "INTERVAL" => interval = val.parse().unwrap_or(1),
            "BYDAY" => byday = val.to_string(),
            "BYHOUR" => byhour = val.to_string(),
            "BYMINUTE" => byminute = val.to_string(),
            "BYMONTHDAY" => bymonthday = val.to_string(),
            "BYMONTH" => bymonth = val.to_string(),
            _ => {}
        }
    }

    let base = if interval == 1 {
        format!("Every {freq}")
    } else {
        format!("Every {interval} {freq}s")
    };

    let mut result = if !byday.is_empty() {
        format!("{base} ({byday})")
    } else {
        base
    };

    if freq == "year" {
        if let Ok(mo) = bymonth.parse::<usize>() {
            if (1..=12).contains(&mo) {
                let month_name = MONTH_NAMES[mo - 1];
                if !bymonthday.is_empty() {
                    result = format!("{result} on {month_name} {bymonthday}");
                } else {
                    result = format!("{result} in {month_name}");
                }
            }
        } else if !bymonthday.is_empty() {
            result = format!("{result} on the {}", ordinal_suffix(&bymonthday));
        }
    } else if freq == "month" && !bymonthday.is_empty() {
        result = format!("{result} on the {}", ordinal_suffix(&bymonthday));
    }

    if !byhour.is_empty() {
        let h: u32 = byhour.parse().unwrap_or(0);
        let m: u32 = byminute.parse().unwrap_or(0);
        let (h12, ampm) = if h == 0 {
            (12, "AM")
        } else if h < 12 {
            (h, "AM")
        } else if h == 12 {
            (12, "PM")
        } else {
            (h - 12, "PM")
        };
        let time_str = if m == 0 {
            format!("{h12} {ampm}")
        } else {
            format!("{h12}:{m:02} {ampm}")
        };
        result = format!("{result} at {time_str}");
    }

    result
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
