use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::schema::project_columns;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(super::ProjectRow, foreign_key = project_id))]
#[diesel(table_name = project_columns)]
pub struct ColumnRow {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub color: String,
    pub position: i32,
    pub is_done: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = project_columns)]
pub struct NewColumn<'a> {
    pub project_id: i64,
    pub name: &'a str,
    pub color: &'a str,
    pub position: i32,
    pub is_done: bool,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = project_columns)]
pub struct ColumnChangeset<'a> {
    pub name: Option<&'a str>,
    pub color: Option<&'a str>,
    pub position: Option<i32>,
    pub is_done: Option<bool>,
}

impl From<ColumnRow> for north_domain::Column {
    fn from(row: ColumnRow) -> Self {
        north_domain::Column {
            id: row.id,
            project_id: row.project_id,
            name: row.name,
            color: row.color,
            position: row.position,
            is_done: row.is_done,
            created_at: row.created_at,
        }
    }
}
