use diesel::prelude::*;

use crate::schema::tags;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = tags)]
pub struct TagRow {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = tags)]
pub struct NewTag<'a> {
    pub user_id: i64,
    pub name: &'a str,
    pub color: &'a str,
}

impl From<TagRow> for north_dto::Tag {
    fn from(row: TagRow) -> Self {
        north_dto::Tag {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            color: row.color,
        }
    }
}

impl From<&TagRow> for north_dto::TagInfo {
    fn from(row: &TagRow) -> Self {
        north_dto::TagInfo {
            name: row.name.clone(),
            color: row.color.clone(),
        }
    }
}
