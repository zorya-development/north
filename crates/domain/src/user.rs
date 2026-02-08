use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(default = "default_review_interval")]
    pub review_interval_days: i16,
    #[serde(default = "default_sequential_limit")]
    pub default_sequential_limit: i16,
    #[serde(default = "default_columns")]
    pub default_columns: Vec<DefaultColumn>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            review_interval_days: default_review_interval(),
            default_sequential_limit: default_sequential_limit(),
            default_columns: default_columns(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultColumn {
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub is_done: bool,
}

fn default_review_interval() -> i16 {
    7
}

fn default_sequential_limit() -> i16 {
    1
}

fn default_columns() -> Vec<DefaultColumn> {
    vec![
        DefaultColumn {
            name: "To Do".into(),
            color: "#6b7280".into(),
            is_done: false,
        },
        DefaultColumn {
            name: "In Progress".into(),
            color: "#3b82f6".into(),
            is_done: false,
        },
        DefaultColumn {
            name: "Done".into(),
            color: "#22c55e".into(),
            is_done: true,
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub settings: UserSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettings {
    pub review_interval_days: Option<i16>,
    pub default_sequential_limit: Option<i16>,
    pub default_columns: Option<Vec<DefaultColumn>>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: User,
}
