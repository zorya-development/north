use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub color: String,
    pub position: i32,
    pub is_done: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateColumn {
    pub name: String,
    pub color: Option<String>,
    pub is_done: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateColumn {
    pub name: Option<String>,
    pub color: Option<String>,
    pub position: Option<i32>,
    pub is_done: Option<bool>,
}
