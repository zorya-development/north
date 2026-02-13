use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub column_id: Option<i64>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWithMeta {
    #[serde(flatten)]
    pub task: Task,
    pub project_title: Option<String>,
    pub column_name: Option<String>,
    pub tags: Vec<crate::TagInfo>,
    pub subtask_count: i64,
    pub completed_subtask_count: i64,
    pub actionable: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub body: Option<String>,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub column_id: Option<i64>,
    pub start_at: Option<DateTime<Utc>>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub body: Option<String>,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub column_id: Option<i64>,
    pub sort_key: Option<String>,
    pub sequential_limit: Option<i16>,
    pub start_at: Option<DateTime<Utc>>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveTask {
    pub column_id: Option<i64>,
    pub sort_key: Option<String>,
    pub parent_id: Option<Option<i64>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct TaskFilter {
    pub project: Option<i64>,
    pub parent: Option<i64>,
    pub column: Option<i64>,
    pub tag: Option<Vec<String>>,
    pub actionable: Option<bool>,
    pub review_due: Option<bool>,
    pub inbox: Option<bool>,
    pub completed: Option<bool>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
