use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::schema::images;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = images)]
pub struct ImageRow {
    pub id: i64,
    pub user_id: i64,
    pub task_id: Option<i64>,
    pub path: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = images)]
pub struct NewImage<'a> {
    pub user_id: i64,
    pub task_id: Option<i64>,
    pub path: &'a str,
    pub filename: &'a str,
    pub content_type: &'a str,
    pub size_bytes: i64,
}
