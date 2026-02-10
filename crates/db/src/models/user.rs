use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::schema::users;
use crate::sql_types::UserRoleMapping;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
pub struct UserRow {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: UserRoleMapping,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: &'a str,
    pub name: &'a str,
    pub role: UserRoleMapping,
    pub settings: serde_json::Value,
}

impl From<UserRow> for north_domain::User {
    fn from(row: UserRow) -> Self {
        let settings: north_domain::UserSettings =
            serde_json::from_value(row.settings).unwrap_or_default();
        north_domain::User {
            id: row.id,
            email: row.email,
            name: row.name,
            role: row.role.into(),
            settings,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
