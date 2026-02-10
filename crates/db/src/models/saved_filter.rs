use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::schema::saved_filters;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = saved_filters)]
pub struct SavedFilterRow {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub query: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = saved_filters)]
pub struct NewSavedFilter<'a> {
    pub user_id: i64,
    pub title: &'a str,
    pub query: &'a str,
    pub position: i32,
}

#[derive(Debug, Default, AsChangeset)]
#[diesel(table_name = saved_filters)]
pub struct SavedFilterChangeset<'a> {
    pub title: Option<&'a str>,
    pub query: Option<&'a str>,
    pub position: Option<i32>,
}

impl From<SavedFilterRow> for north_domain::SavedFilter {
    fn from(row: SavedFilterRow) -> Self {
        north_domain::SavedFilter {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            query: row.query,
            position: row.position,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
