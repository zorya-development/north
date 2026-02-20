use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::RecurrenceType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
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
    pub recurrence_type: Option<RecurrenceType>,
    pub recurrence_rule: Option<String>,
    #[serde(default)]
    pub project_title: Option<String>,
    #[serde(default)]
    pub tags: Vec<crate::TagInfo>,
    #[serde(default)]
    pub subtask_count: i64,
    #[serde(default)]
    pub completed_subtask_count: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub body: Option<String>,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub sort_key: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub body: Option<Option<String>>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub project_id: Option<Option<i64>>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub parent_id: Option<Option<i64>>,

    pub sort_key: Option<String>,
    pub sequential_limit: Option<i16>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub start_at: Option<Option<DateTime<Utc>>>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub due_date: Option<Option<NaiveDate>>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub completed_at: Option<Option<DateTime<Utc>>>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub reviewed_at: Option<Option<NaiveDate>>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub recurrence_type: Option<Option<RecurrenceType>>,

    #[serde(
        default,
        skip_serializing_if = "crate::serde_helpers::is_none_outer",
        with = "crate::serde_helpers::double_option"
    )]
    pub recurrence_rule: Option<Option<String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TaskFilter {
    pub project: Option<i64>,
    pub parent: Option<i64>,
    pub tag: Option<Vec<String>>,
    pub review_due: Option<bool>,
    pub inbox: Option<bool>,
    pub completed: Option<bool>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
