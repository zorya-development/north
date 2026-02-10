use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFilter {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub query: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSavedFilter {
    pub title: String,
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSavedFilter {
    pub title: Option<String>,
    pub query: Option<String>,
    pub position: Option<i32>,
}
