use crate::filter::dsl::FilterField;
use north_dto::Task;

pub struct TaskFieldRegistry;

impl TaskFieldRegistry {
    /// Exhaustive destructure — NO `..` rest pattern.
    /// Fails to compile if Task gains/loses a field.
    #[allow(unused_variables)]
    fn _assert_exhaustive(task: &Task) {
        let Task {
            id,
            project_id,
            parent_id,
            user_id,
            title, // FilterField::Title
            body,  // FilterField::Body
            sort_key,
            sequential_limit,
            start_at,     // FilterField::StartAt
            due_date,     // FilterField::DueDate
            completed_at, // FilterField::Status (derived)
            reviewed_at,
            created_at, // FilterField::Created
            updated_at, // FilterField::Updated
            recurrence_type,
            recurrence_rule,
            project_title, // FilterField::Project (enriched)
            tags,          // FilterField::Tags (enriched)
            subtask_count,
            completed_subtask_count,
            actionable,
        } = task;
    }

    pub fn from_str_ci(s: &str) -> Option<FilterField> {
        match s.to_lowercase().as_str() {
            "title" => Some(FilterField::Title),
            "body" => Some(FilterField::Body),
            "project" => Some(FilterField::Project),
            "tags" | "tag" => Some(FilterField::Tags),
            "status" => Some(FilterField::Status),
            "due_date" | "due" => Some(FilterField::DueDate),
            "start_at" | "start" => Some(FilterField::StartAt),
            "created" | "created_at" => Some(FilterField::Created),
            "updated" | "updated_at" => Some(FilterField::Updated),
            _ => None,
        }
    }

    pub fn field_names() -> &'static [&'static str] {
        &[
            "title", "body", "project", "tags", "status", "due_date", "start_at", "created",
            "updated",
        ]
    }
}
