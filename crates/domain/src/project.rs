use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Column;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectViewType {
    List,
    Kanban,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub view_type: ProjectViewType,
    pub position: i32,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectWithColumns {
    #[serde(flatten)]
    pub project: Project,
    pub columns: Vec<Column>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub title: String,
    pub description: Option<String>,
    pub view_type: Option<ProjectViewType>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub title: Option<String>,
    pub description: Option<String>,
    pub view_type: Option<ProjectViewType>,
    pub position: Option<i32>,
    pub archived: Option<bool>,
}
