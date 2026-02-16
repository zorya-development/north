use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::schema::projects;
use crate::sql_types::{ProjectStatusMapping, ProjectViewTypeMapping};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = projects)]
pub struct ProjectRow {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub view_type: ProjectViewTypeMapping,
    pub position: i32,
    pub color: String,
    pub status: ProjectStatusMapping,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = projects)]
pub struct NewProject<'a> {
    pub user_id: i64,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub view_type: ProjectViewTypeMapping,
    pub position: i32,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = projects)]
pub struct ProjectChangeset<'a> {
    pub title: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub view_type: Option<ProjectViewTypeMapping>,
    pub position: Option<i32>,
    pub color: Option<&'a str>,
    pub status: Option<ProjectStatusMapping>,
}

impl From<ProjectRow> for north_dto::Project {
    fn from(row: ProjectRow) -> Self {
        north_dto::Project {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            description: row.description,
            view_type: row.view_type.into(),
            position: row.position,
            color: row.color,
            status: row.status.into(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
