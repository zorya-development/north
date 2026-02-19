use leptos::prelude::*;
use north_dto::{Frequency, RecurrenceRule, RecurrenceType, Weekday};
#[cfg(feature = "hydrate")]
use north_repositories::SettingsRepository;

#[derive(Clone, Copy)]
pub struct RecurrenceController {
    pub rule: RwSignal<RecurrenceRule>,
    pub recurrence_type: RwSignal<RecurrenceType>,
    pub timezone: Signal<String>,
    is_custom: RwSignal<bool>,
}

impl RecurrenceController {
    pub fn new(existing_type: Option<RecurrenceType>, existing_rule: Option<String>) -> Self {
        let parsed = existing_rule
            .as_deref()
            .and_then(RecurrenceRule::parse)
            .unwrap_or_default();

        let is_custom = RwSignal::new(parsed.interval != 1);
        let rule = RwSignal::new(parsed);
        let recurrence_type = RwSignal::new(existing_type.unwrap_or(RecurrenceType::Scheduled));

        let tz = RwSignal::new(String::from("UTC"));
        #[cfg(feature = "hydrate")]
        {
            Effect::new(move |_| {
                leptos::task::spawn_local(async move {
                    if let Ok(settings) = SettingsRepository::get().await {
                        tz.set(settings.timezone);
                    }
                });
            });
        }

        Self {
            rule,
            recurrence_type,
            timezone: Signal::derive(move || tz.get()),
            is_custom,
        }
    }

    pub fn build_result(&self) -> (Option<RecurrenceType>, Option<String>) {
        let rt = self.recurrence_type.get_untracked();
        let rule = self.rule.get_untracked();
        (Some(rt), Some(rule.to_rrule_string()))
    }

    pub fn summary(&self) -> Signal<String> {
        let rule = self.rule;
        Signal::derive(move || rule.get().summarize())
    }

    pub fn freq(&self) -> Signal<Frequency> {
        let rule = self.rule;
        Signal::derive(move || rule.get().freq)
    }

    pub fn interval(&self) -> Signal<u32> {
        let rule = self.rule;
        Signal::derive(move || rule.get().interval)
    }

    pub fn is_day_selected(&self, day: Weekday) -> Signal<bool> {
        let rule = self.rule;
        Signal::derive(move || rule.get().by_day.contains(&day))
    }

    pub fn time_str(&self) -> Signal<String> {
        let rule = self.rule;
        Signal::derive(move || rule.get().time_str())
    }

    pub fn by_month_day_str(&self) -> Signal<String> {
        let rule = self.rule;
        Signal::derive(move || {
            rule.get()
                .by_month_day
                .map(|d| d.to_string())
                .unwrap_or_default()
        })
    }

    pub fn by_month_str(&self) -> Signal<String> {
        let rule = self.rule;
        Signal::derive(move || {
            rule.get()
                .by_month
                .map(|m| m.to_string())
                .unwrap_or_default()
        })
    }

    pub fn is_custom(&self) -> Signal<bool> {
        let is_custom = self.is_custom;
        Signal::derive(move || is_custom.get())
    }

    pub fn set_freq(&self, freq: Frequency) {
        self.rule.update(|r| r.freq = freq);
    }

    pub fn set_interval(&self, interval: u32) {
        self.rule.update(|r| r.interval = interval);
    }

    pub fn toggle_day(&self, day: Weekday) {
        self.rule.update(|r| {
            if !r.by_day.remove(&day) {
                r.by_day.insert(day);
            }
        });
    }

    pub fn set_time(&self, time_str: &str) {
        let s = time_str.to_string();
        self.rule.update(|r| r.set_time_str(&s));
    }

    pub fn set_month_day(&self, val: &str) {
        let md = val.parse::<u32>().ok();
        self.rule.update(|r| r.by_month_day = md);
    }

    pub fn set_month(&self, val: &str) {
        let m = val.parse::<u32>().ok();
        self.rule.update(|r| r.by_month = m);
    }

    pub fn select_preset(&self, preset: &str) {
        match preset {
            "daily" => {
                self.set_freq(Frequency::Daily);
                self.set_interval(1);
                self.is_custom.set(false);
            }
            "weekly" => {
                self.set_freq(Frequency::Weekly);
                self.set_interval(1);
                self.is_custom.set(false);
            }
            "monthly" => {
                self.set_freq(Frequency::Monthly);
                self.set_interval(1);
                self.is_custom.set(false);
            }
            "yearly" => {
                self.set_freq(Frequency::Yearly);
                self.set_interval(1);
                self.is_custom.set(false);
            }
            "custom" => {
                self.is_custom.set(true);
            }
            _ => {}
        }
    }

    pub fn set_recurrence_type(&self, rt: RecurrenceType) {
        self.recurrence_type.set(rt);
    }

    pub fn active_preset(&self) -> Signal<&'static str> {
        let freq = self.freq();
        let is_custom = self.is_custom;
        Signal::derive(move || {
            if is_custom.get() {
                return "custom";
            }
            match freq.get() {
                Frequency::Daily => "daily",
                Frequency::Weekly => "weekly",
                Frequency::Monthly => "monthly",
                Frequency::Yearly => "yearly",
            }
        })
    }

    pub fn set_freq_from_code(&self, code: &str) {
        if let Some(f) = Frequency::from_code(code) {
            self.set_freq(f);
        }
    }
}
