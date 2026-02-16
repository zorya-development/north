use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslSuggestion {
    pub label: String,
    pub value: String,
    pub color: String,
    pub start: usize,
}
