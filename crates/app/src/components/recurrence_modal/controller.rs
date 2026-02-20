use north_dto::RecurrenceType;
use north_stores::use_app_store;

use crate::libs::ReactiveRecurrenceRule;

#[derive(Clone, Copy)]
pub struct RecurrenceModalController {
    pub rule: ReactiveRecurrenceRule,
}

impl RecurrenceModalController {
    pub fn new(existing_type: Option<RecurrenceType>, existing_rule: Option<String>) -> Self {
        let app_store = use_app_store();
        let timezone = app_store.settings.timezone();
        let rule = ReactiveRecurrenceRule::from_str(existing_type, existing_rule, timezone);
        Self { rule }
    }

    pub fn save_result(&self) -> (Option<RecurrenceType>, Option<String>) {
        self.rule.to_result()
    }

    pub fn remove_result() -> (Option<RecurrenceType>, Option<String>) {
        (None, None)
    }
}
