use diesel::prelude::*;

use crate::schema::task_tags;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(super::TaskRow, foreign_key = task_id))]
#[diesel(belongs_to(super::TagRow, foreign_key = tag_id))]
#[diesel(table_name = task_tags)]
#[diesel(primary_key(task_id, tag_id))]
pub struct TaskTagRow {
    pub task_id: i64,
    pub tag_id: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = task_tags)]
pub struct NewTaskTag {
    pub task_id: i64,
    pub tag_id: i64,
}
