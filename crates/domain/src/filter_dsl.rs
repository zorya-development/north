use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterQuery {
    pub expression: Option<FilterExpr>,
    pub order_by: Option<OrderBy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterExpr {
    Condition(Condition),
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
    Not(Box<FilterExpr>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    pub field: FilterField,
    pub op: FilterOp,
    pub value: FilterValue,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterField {
    Title,
    Body,
    Project,
    Tags,
    Status,
    DueDate,
    StartAt,
    Column,
    Created,
    Updated,
}

impl FilterField {
    pub fn from_str_ci(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "title" => Some(Self::Title),
            "body" => Some(Self::Body),
            "project" => Some(Self::Project),
            "tags" | "tag" => Some(Self::Tags),
            "status" => Some(Self::Status),
            "due_date" | "due" => Some(Self::DueDate),
            "start_at" | "start" => Some(Self::StartAt),
            "column" | "col" => Some(Self::Column),
            "created" | "created_at" => Some(Self::Created),
            "updated" | "updated_at" => Some(Self::Updated),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterOp {
    Eq,
    Ne,
    GlobMatch,
    GlobNotMatch,
    Gt,
    Lt,
    Gte,
    Lte,
    Is,
    IsNot,
    In,
    NotIn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Array(Vec<FilterValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBy {
    pub field: FilterField,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Asc
    }
}
