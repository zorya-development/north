use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecurrenceType {
    Scheduled,
    AfterCompletion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Frequency {
    pub fn code(self) -> &'static str {
        match self {
            Self::Daily => "DAILY",
            Self::Weekly => "WEEKLY",
            Self::Monthly => "MONTHLY",
            Self::Yearly => "YEARLY",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s {
            "DAILY" => Some(Self::Daily),
            "WEEKLY" => Some(Self::Weekly),
            "MONTHLY" => Some(Self::Monthly),
            "YEARLY" => Some(Self::Yearly),
            _ => None,
        }
    }

    pub fn unit_singular(self) -> &'static str {
        match self {
            Self::Daily => "day",
            Self::Weekly => "week",
            Self::Monthly => "month",
            Self::Yearly => "year",
        }
    }

    pub fn unit_plural(self) -> &'static str {
        match self {
            Self::Daily => "days",
            Self::Weekly => "weeks",
            Self::Monthly => "months",
            Self::Yearly => "years",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Weekday {
    Mo,
    Tu,
    We,
    Th,
    Fr,
    Sa,
    Su,
}

impl Weekday {
    pub const ALL: [Weekday; 7] = [
        Self::Mo,
        Self::Tu,
        Self::We,
        Self::Th,
        Self::Fr,
        Self::Sa,
        Self::Su,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::Mo => "MO",
            Self::Tu => "TU",
            Self::We => "WE",
            Self::Th => "TH",
            Self::Fr => "FR",
            Self::Sa => "SA",
            Self::Su => "SU",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s {
            "MO" => Some(Self::Mo),
            "TU" => Some(Self::Tu),
            "WE" => Some(Self::We),
            "TH" => Some(Self::Th),
            "FR" => Some(Self::Fr),
            "SA" => Some(Self::Sa),
            "SU" => Some(Self::Su),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Mo => "Mo",
            Self::Tu => "Tu",
            Self::We => "We",
            Self::Th => "Th",
            Self::Fr => "Fr",
            Self::Sa => "Sa",
            Self::Su => "Su",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub freq: Frequency,
    pub interval: u32,
    pub by_day: BTreeSet<Weekday>,
    pub by_hour: Option<u32>,
    pub by_minute: Option<u32>,
    pub by_month_day: Option<u32>,
    pub by_month: Option<u32>,
}

impl Default for RecurrenceRule {
    fn default() -> Self {
        Self {
            freq: Frequency::Daily,
            interval: 1,
            by_day: BTreeSet::new(),
            by_hour: Some(9),
            by_minute: Some(0),
            by_month_day: None,
            by_month: None,
        }
    }
}

impl RecurrenceRule {
    pub fn parse(s: &str) -> Option<Self> {
        let mut freq = None;
        let mut interval = 1u32;
        let mut by_day = BTreeSet::new();
        let mut by_hour = None;
        let mut by_minute = None;
        let mut by_month_day = None;
        let mut by_month = None;

        for part in s.split(';') {
            let mut kv = part.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim();
            let val = kv.next().unwrap_or("").trim();
            match key {
                "FREQ" => freq = Frequency::from_code(val),
                "INTERVAL" => interval = val.parse().unwrap_or(1),
                "BYDAY" => {
                    for day_code in val.split(',') {
                        if let Some(d) = Weekday::from_code(day_code.trim()) {
                            by_day.insert(d);
                        }
                    }
                }
                "BYHOUR" => by_hour = val.parse().ok(),
                "BYMINUTE" => by_minute = val.parse().ok(),
                "BYMONTHDAY" => by_month_day = val.parse().ok(),
                "BYMONTH" => by_month = val.parse().ok(),
                _ => {}
            }
        }

        Some(Self {
            freq: freq?,
            interval,
            by_day,
            by_hour,
            by_minute,
            by_month_day,
            by_month,
        })
    }

    pub fn to_rrule_string(&self) -> String {
        let mut rule = format!("FREQ={};INTERVAL={}", self.freq.code(), self.interval);

        if self.freq == Frequency::Weekly && !self.by_day.is_empty() {
            let days: Vec<&str> = self.by_day.iter().map(|d| d.code()).collect();
            rule.push_str(&format!(";BYDAY={}", days.join(",")));
        }

        if let Some(md) = self.by_month_day {
            if self.freq == Frequency::Monthly || self.freq == Frequency::Yearly {
                rule.push_str(&format!(";BYMONTHDAY={md}"));
            }
        }

        if self.freq == Frequency::Yearly {
            if let Some(m) = self.by_month {
                rule.push_str(&format!(";BYMONTH={m}"));
            }
        }

        if let Some(h) = self.by_hour {
            rule.push_str(&format!(";BYHOUR={h}"));
            rule.push_str(&format!(";BYMINUTE={}", self.by_minute.unwrap_or(0)));
        }

        rule
    }

    pub fn summarize(&self) -> String {
        let base = if self.interval == 1 {
            format!("Every {}", self.freq.unit_singular())
        } else {
            format!("Every {} {}", self.interval, self.freq.unit_plural())
        };

        let mut result = base;

        if self.freq == Frequency::Weekly && !self.by_day.is_empty() {
            let days: Vec<&str> = self.by_day.iter().map(|d| d.code()).collect();
            result = format!("{result} ({})", days.join(","));
        }

        if self.freq == Frequency::Yearly {
            if let Some(m) = self.by_month {
                if (1..=12).contains(&m) {
                    let month_name = MONTH_NAMES[m as usize - 1];
                    if let Some(md) = self.by_month_day {
                        result = format!("{result} on {month_name} {md}");
                    } else {
                        result = format!("{result} in {month_name}");
                    }
                }
            } else if let Some(md) = self.by_month_day {
                result = format!("{result} on the {}", ordinal_suffix(md));
            }
        } else if self.freq == Frequency::Monthly {
            if let Some(md) = self.by_month_day {
                result = format!("{result} on the {}", ordinal_suffix(md));
            }
        }

        if let Some(h) = self.by_hour {
            let m = self.by_minute.unwrap_or(0);
            result = format!("{result} at {}", format_time_12h(h, m));
        }

        result
    }

    pub fn time_str(&self) -> String {
        let h = self.by_hour.unwrap_or(9);
        let m = self.by_minute.unwrap_or(0);
        format!("{h:02}:{m:02}")
    }

    pub fn set_time_str(&mut self, s: &str) {
        let mut parts = s.split(':');
        if let (Some(h), Some(m)) = (
            parts.next().and_then(|v| v.parse::<u32>().ok()),
            parts.next().and_then(|v| v.parse::<u32>().ok()),
        ) {
            self.by_hour = Some(h);
            self.by_minute = Some(m);
        }
    }
}

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn ordinal_suffix(n: u32) -> String {
    let suffix = match n % 10 {
        1 if n % 100 != 11 => "st",
        2 if n % 100 != 12 => "nd",
        3 if n % 100 != 13 => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn format_time_12h(h: u32, m: u32) -> String {
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
        format!("{h12} {ampm}")
    } else {
        format!("{h12}:{m:02} {ampm}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_daily() {
        let rule = RecurrenceRule::parse("FREQ=DAILY;INTERVAL=1;BYHOUR=9;BYMINUTE=0").unwrap();
        assert_eq!(rule.freq, Frequency::Daily);
        assert_eq!(rule.interval, 1);
        assert_eq!(rule.by_hour, Some(9));
        assert_eq!(rule.by_minute, Some(0));
    }

    #[test]
    fn parse_weekly_with_days() {
        let rule =
            RecurrenceRule::parse("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE,FR;BYHOUR=10;BYMINUTE=30")
                .unwrap();
        assert_eq!(rule.freq, Frequency::Weekly);
        assert_eq!(rule.interval, 2);
        assert!(rule.by_day.contains(&Weekday::Mo));
        assert!(rule.by_day.contains(&Weekday::We));
        assert!(rule.by_day.contains(&Weekday::Fr));
        assert_eq!(rule.by_day.len(), 3);
        assert_eq!(rule.by_hour, Some(10));
        assert_eq!(rule.by_minute, Some(30));
    }

    #[test]
    fn parse_monthly() {
        let rule =
            RecurrenceRule::parse("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=15;BYHOUR=8;BYMINUTE=0")
                .unwrap();
        assert_eq!(rule.freq, Frequency::Monthly);
        assert_eq!(rule.by_month_day, Some(15));
    }

    #[test]
    fn parse_yearly() {
        let rule = RecurrenceRule::parse(
            "FREQ=YEARLY;INTERVAL=1;BYMONTHDAY=25;BYMONTH=12;BYHOUR=9;BYMINUTE=0",
        )
        .unwrap();
        assert_eq!(rule.freq, Frequency::Yearly);
        assert_eq!(rule.by_month_day, Some(25));
        assert_eq!(rule.by_month, Some(12));
    }

    #[test]
    fn parse_missing_freq_returns_none() {
        assert!(RecurrenceRule::parse("INTERVAL=1;BYHOUR=9").is_none());
    }

    #[test]
    fn parse_invalid_freq_returns_none() {
        assert!(RecurrenceRule::parse("FREQ=BIWEEKLY;INTERVAL=1").is_none());
    }

    #[test]
    fn roundtrip_daily() {
        let rule = RecurrenceRule {
            freq: Frequency::Daily,
            interval: 3,
            by_hour: Some(14),
            by_minute: Some(30),
            ..Default::default()
        };
        let s = rule.to_rrule_string();
        let parsed = RecurrenceRule::parse(&s).unwrap();
        assert_eq!(parsed.freq, Frequency::Daily);
        assert_eq!(parsed.interval, 3);
        assert_eq!(parsed.by_hour, Some(14));
        assert_eq!(parsed.by_minute, Some(30));
    }

    #[test]
    fn roundtrip_weekly() {
        let mut by_day = BTreeSet::new();
        by_day.insert(Weekday::Mo);
        by_day.insert(Weekday::Fr);
        let rule = RecurrenceRule {
            freq: Frequency::Weekly,
            interval: 1,
            by_day,
            by_hour: Some(9),
            by_minute: Some(0),
            ..Default::default()
        };
        let s = rule.to_rrule_string();
        let parsed = RecurrenceRule::parse(&s).unwrap();
        assert_eq!(parsed.freq, Frequency::Weekly);
        assert!(parsed.by_day.contains(&Weekday::Mo));
        assert!(parsed.by_day.contains(&Weekday::Fr));
        assert_eq!(parsed.by_day.len(), 2);
    }

    #[test]
    fn roundtrip_yearly() {
        let rule = RecurrenceRule {
            freq: Frequency::Yearly,
            interval: 1,
            by_month_day: Some(25),
            by_month: Some(12),
            by_hour: Some(9),
            by_minute: Some(0),
            ..Default::default()
        };
        let s = rule.to_rrule_string();
        let parsed = RecurrenceRule::parse(&s).unwrap();
        assert_eq!(parsed.freq, Frequency::Yearly);
        assert_eq!(parsed.by_month_day, Some(25));
        assert_eq!(parsed.by_month, Some(12));
    }

    #[test]
    fn summarize_daily() {
        let rule = RecurrenceRule {
            freq: Frequency::Daily,
            interval: 1,
            by_hour: Some(9),
            by_minute: Some(0),
            ..Default::default()
        };
        assert_eq!(rule.summarize(), "Every day at 9 AM");
    }

    #[test]
    fn summarize_daily_plural() {
        let rule = RecurrenceRule {
            freq: Frequency::Daily,
            interval: 3,
            by_hour: Some(14),
            by_minute: Some(30),
            ..Default::default()
        };
        assert_eq!(rule.summarize(), "Every 3 days at 2:30 PM");
    }

    #[test]
    fn summarize_weekly_with_days() {
        let mut by_day = BTreeSet::new();
        by_day.insert(Weekday::Mo);
        by_day.insert(Weekday::We);
        by_day.insert(Weekday::Fr);
        let rule = RecurrenceRule {
            freq: Frequency::Weekly,
            interval: 1,
            by_day,
            by_hour: Some(10),
            by_minute: Some(0),
            ..Default::default()
        };
        assert_eq!(rule.summarize(), "Every week (MO,WE,FR) at 10 AM");
    }

    #[test]
    fn summarize_monthly() {
        let rule = RecurrenceRule {
            freq: Frequency::Monthly,
            interval: 1,
            by_month_day: Some(1),
            by_hour: Some(9),
            by_minute: Some(0),
            ..Default::default()
        };
        assert_eq!(rule.summarize(), "Every month on the 1st at 9 AM");
    }

    #[test]
    fn summarize_yearly_with_month() {
        let rule = RecurrenceRule {
            freq: Frequency::Yearly,
            interval: 1,
            by_month_day: Some(25),
            by_month: Some(12),
            by_hour: Some(9),
            by_minute: Some(0),
            ..Default::default()
        };
        assert_eq!(rule.summarize(), "Every year on Dec 25 at 9 AM");
    }

    #[test]
    fn time_str_roundtrip() {
        let mut rule = RecurrenceRule::default();
        rule.set_time_str("14:30");
        assert_eq!(rule.time_str(), "14:30");
        assert_eq!(rule.by_hour, Some(14));
        assert_eq!(rule.by_minute, Some(30));
    }

    #[test]
    fn time_str_default() {
        let rule = RecurrenceRule::default();
        assert_eq!(rule.time_str(), "09:00");
    }
}
