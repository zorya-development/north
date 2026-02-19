use leptos::prelude::*;
use north_dto::RecurrenceType;
#[cfg(feature = "hydrate")]
use north_repositories::SettingsRepository;

use super::view::RecurrenceModalView;

#[component]
pub fn RecurrenceModal(
    recurrence_type: Option<RecurrenceType>,
    recurrence_rule: Option<String>,
    on_save: Callback<(Option<RecurrenceType>, Option<String>)>,
    on_close: Callback<()>,
) -> impl IntoView {
    let parsed = parse_existing(recurrence_type, recurrence_rule.as_deref());

    let mode = RwSignal::new(parsed.mode);
    let freq = RwSignal::new(parsed.freq);
    let interval = RwSignal::new(parsed.interval);
    let monday = RwSignal::new(parsed.byday.contains("MO"));
    let tuesday = RwSignal::new(parsed.byday.contains("TU"));
    let wednesday = RwSignal::new(parsed.byday.contains("WE"));
    let thursday = RwSignal::new(parsed.byday.contains("TH"));
    let friday = RwSignal::new(parsed.byday.contains("FR"));
    let saturday = RwSignal::new(parsed.byday.contains("SA"));
    let sunday = RwSignal::new(parsed.byday.contains("SU"));
    let time = RwSignal::new(parsed.time);
    let monthday = RwSignal::new(parsed.monthday);
    let month = RwSignal::new(parsed.month);
    let timezone = RwSignal::new(String::from("UTC"));

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            leptos::task::spawn_local(async move {
                if let Ok(settings) = SettingsRepository::get().await {
                    timezone.set(settings.timezone);
                }
            });
        });
    }

    view! {
        <RecurrenceModalView
            mode=mode
            freq=freq
            interval=interval
            monday=monday
            tuesday=tuesday
            wednesday=wednesday
            thursday=thursday
            friday=friday
            saturday=saturday
            sunday=sunday
            time=time
            monthday=monthday
            month=month
            timezone=Signal::derive(move || timezone.get())
            on_save=Callback::new(move |()| {
                let m = mode.get_untracked();
                let rec_type = if m == "scheduled" {
                    RecurrenceType::Scheduled
                } else {
                    RecurrenceType::AfterCompletion
                };
                let f = freq.get_untracked();
                let i = interval.get_untracked();
                let mut rule = format!("FREQ={f};INTERVAL={i}");
                if f == "WEEKLY" {
                    let mut days = Vec::new();
                    if monday.get_untracked() { days.push("MO"); }
                    if tuesday.get_untracked() { days.push("TU"); }
                    if wednesday.get_untracked() { days.push("WE"); }
                    if thursday.get_untracked() { days.push("TH"); }
                    if friday.get_untracked() { days.push("FR"); }
                    if saturday.get_untracked() { days.push("SA"); }
                    if sunday.get_untracked() { days.push("SU"); }
                    if !days.is_empty() {
                        rule.push_str(&format!(";BYDAY={}", days.join(",")));
                    }
                }
                if f == "MONTHLY" || f == "YEARLY" {
                    let md = monthday.get_untracked();
                    if !md.is_empty() {
                        rule.push_str(&format!(";BYMONTHDAY={md}"));
                    }
                }
                if f == "YEARLY" {
                    let mo = month.get_untracked();
                    if !mo.is_empty() {
                        rule.push_str(&format!(";BYMONTH={mo}"));
                    }
                }
                let t = time.get_untracked();
                if !t.is_empty() {
                    if let Some((h, m)) = parse_time_str(&t) {
                        rule.push_str(&format!(";BYHOUR={h};BYMINUTE={m}"));
                    }
                }
                on_save.run((Some(rec_type), Some(rule)));
            })
            on_remove=Callback::new(move |()| {
                on_save.run((None, None));
            })
            on_close=on_close
        />
    }
}

struct ParsedRule {
    mode: String,
    freq: String,
    interval: String,
    byday: String,
    time: String,
    monthday: String,
    month: String,
}

fn parse_existing(rec_type: Option<RecurrenceType>, rec_rule: Option<&str>) -> ParsedRule {
    let mode = match rec_type {
        Some(RecurrenceType::Scheduled) => "scheduled",
        Some(RecurrenceType::AfterCompletion) => "after_completion",
        None => "scheduled",
    };

    let mut freq = "DAILY";
    let mut interval = "1";
    let mut byday = String::new();
    let mut byhour = String::new();
    let mut byminute = String::new();
    let mut bymonthday = String::new();
    let mut bymonth = String::new();

    if let Some(rule) = rec_rule {
        for part in rule.split(';') {
            let mut kv = part.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim();
            let val = kv.next().unwrap_or("").trim();
            match key {
                "FREQ" => {
                    freq = match val {
                        "DAILY" | "WEEKLY" | "MONTHLY" | "YEARLY" => val,
                        _ => freq,
                    }
                }
                "INTERVAL" => interval = val,
                "BYDAY" => byday = val.to_string(),
                "BYHOUR" => byhour = val.to_string(),
                "BYMINUTE" => byminute = val.to_string(),
                "BYMONTHDAY" => bymonthday = val.to_string(),
                "BYMONTH" => bymonth = val.to_string(),
                _ => {}
            }
        }
    }

    let time = if !byhour.is_empty() {
        let h: u32 = byhour.parse().unwrap_or(0);
        let m: u32 = byminute.parse().unwrap_or(0);
        format!("{h:02}:{m:02}")
    } else {
        String::from("09:00")
    };

    ParsedRule {
        mode: mode.to_string(),
        freq: freq.to_string(),
        interval: interval.to_string(),
        byday,
        time,
        monthday: bymonthday,
        month: bymonth,
    }
}

fn parse_time_str(t: &str) -> Option<(u32, u32)> {
    let mut parts = t.split(':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    Some((h, m))
}
