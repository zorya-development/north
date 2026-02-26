use chrono::{DateTime, NaiveDate, Utc};
use north_dto::{RecurrenceRule, RecurrenceType};
use north_dto::{TagInfo, Task};

#[derive(Debug, Clone, PartialEq)]
pub struct Recurrence {
    pub recurrence_type: RecurrenceType,
    pub rule: RecurrenceRule,
}

impl Recurrence {
    pub fn summarize(&self) -> String {
        self.rule.summarize()
    }

    pub fn rule_string(&self) -> String {
        self.rule.to_rrule_string()
    }

    pub fn default_rule() -> (RecurrenceType, String) {
        let rule = RecurrenceRule::default();
        (RecurrenceType::Scheduled, rule.to_rrule_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskModel {
    pub id: i64,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub user_id: i64,
    pub title: String,
    pub body: Option<String>,
    pub sort_key: String,
    pub sequential_limit: i16,
    pub start_at: Option<DateTime<Utc>>,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub recurrence: Option<Recurrence>,
    pub is_url_fetching: Option<DateTime<Utc>>,
    pub someday: bool,
    pub project_title: Option<String>,
    pub tags: Vec<TagInfo>,
    pub subtask_count: i64,
    pub completed_subtask_count: i64,
}

impl From<Task> for TaskModel {
    fn from(t: Task) -> Self {
        let recurrence = t.recurrence_type.and_then(|rt| {
            t.recurrence_rule
                .as_deref()
                .and_then(RecurrenceRule::parse)
                .map(|rule| Recurrence {
                    recurrence_type: rt,
                    rule,
                })
        });

        Self {
            id: t.id,
            project_id: t.project_id,
            parent_id: t.parent_id,
            user_id: t.user_id,
            title: t.title,
            body: t.body,
            sort_key: t.sort_key,
            sequential_limit: t.sequential_limit,
            start_at: t.start_at,
            due_date: t.due_date,
            completed_at: t.completed_at,
            reviewed_at: t.reviewed_at,
            created_at: t.created_at,
            updated_at: t.updated_at,
            recurrence,
            is_url_fetching: t.is_url_fetching,
            someday: t.someday,
            project_title: t.project_title,
            tags: t.tags,
            subtask_count: t.subtask_count,
            completed_subtask_count: t.completed_subtask_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dto_task() -> Task {
        Task {
            id: 1,
            project_id: None,
            parent_id: None,
            user_id: 1,
            title: "Test".to_string(),
            body: None,
            sort_key: "a".to_string(),
            sequential_limit: 0,
            start_at: None,
            due_date: None,
            completed_at: None,
            reviewed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            recurrence_type: None,
            recurrence_rule: None,
            is_url_fetching: None,
            someday: false,
            project_title: None,
            tags: vec![],
            subtask_count: 0,
            completed_subtask_count: 0,
        }
    }

    #[test]
    fn no_recurrence() {
        let dto = make_dto_task();
        let model = TaskModel::from(dto);
        assert!(model.recurrence.is_none());
    }

    #[test]
    fn valid_recurrence() {
        let mut dto = make_dto_task();
        dto.recurrence_type = Some(RecurrenceType::Scheduled);
        dto.recurrence_rule = Some("FREQ=DAILY;INTERVAL=1;BYHOUR=9;BYMINUTE=0".to_string());

        let model = TaskModel::from(dto);
        let rec = model.recurrence.expect("should have recurrence");
        assert_eq!(rec.recurrence_type, RecurrenceType::Scheduled);
        assert_eq!(rec.rule.freq, north_dto::Frequency::Daily);
        assert_eq!(rec.summarize(), "Every day at 9 AM");
        assert_eq!(
            rec.rule_string(),
            "FREQ=DAILY;INTERVAL=1;BYHOUR=9;BYMINUTE=0"
        );
    }

    #[test]
    fn type_present_but_rule_missing() {
        let mut dto = make_dto_task();
        dto.recurrence_type = Some(RecurrenceType::AfterCompletion);
        dto.recurrence_rule = None;

        let model = TaskModel::from(dto);
        assert!(model.recurrence.is_none());
    }

    #[test]
    fn type_present_but_rule_invalid() {
        let mut dto = make_dto_task();
        dto.recurrence_type = Some(RecurrenceType::Scheduled);
        dto.recurrence_rule = Some("INVALID_RULE".to_string());

        let model = TaskModel::from(dto);
        assert!(model.recurrence.is_none());
    }
}
