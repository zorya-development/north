use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;

use crate::schema::tasks;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(super::ProjectRow, foreign_key = project_id))]
#[diesel(table_name = tasks)]
pub struct TaskRow {
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
}

#[derive(Debug, Insertable)]
#[diesel(table_name = tasks)]
pub struct NewTask<'a> {
    pub user_id: i64,
    pub title: &'a str,
    pub body: Option<&'a str>,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub sort_key: &'a str,
    pub start_at: Option<DateTime<Utc>>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Default, AsChangeset)]
#[diesel(table_name = tasks)]
pub struct TaskChangeset<'a> {
    pub title: Option<&'a str>,
    pub body: Option<Option<&'a str>>,
    pub project_id: Option<Option<i64>>,
    pub parent_id: Option<Option<i64>>,
    pub sort_key: Option<&'a str>,
    pub sequential_limit: Option<i16>,
    pub start_at: Option<Option<DateTime<Utc>>>,
    pub due_date: Option<Option<NaiveDate>>,
    pub completed_at: Option<Option<DateTime<Utc>>>,
    pub reviewed_at: Option<Option<NaiveDate>>,
}

impl From<TaskRow> for north_domain::Task {
    fn from(row: TaskRow) -> Self {
        north_domain::Task {
            id: row.id,
            project_id: row.project_id,
            parent_id: row.parent_id,
            user_id: row.user_id,
            title: row.title,
            body: row.body,
            sort_key: row.sort_key,
            sequential_limit: row.sequential_limit,
            start_at: row.start_at,
            due_date: row.due_date,
            completed_at: row.completed_at,
            reviewed_at: row.reviewed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
