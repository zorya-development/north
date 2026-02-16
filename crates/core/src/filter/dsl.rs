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
    Created,
    Updated,
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}
