use leptos::prelude::*;

/// Provided by pages/containers. TTL renders the toolbar from this.
/// When `enabled` is false, no toolbar is rendered.
#[derive(Clone)]
pub struct ToolbarConfig {
    pub enabled: bool,
    pub show_add_task: bool,
    pub completed: Option<CompletedToggle>,
    pub actionable: Option<ActionableToggle>,
}

impl ToolbarConfig {
    /// Returns a config that disables the toolbar entirely.
    pub fn none() -> Self {
        Self {
            enabled: false,
            show_add_task: false,
            completed: None,
            actionable: None,
        }
    }
}

#[derive(Clone)]
pub struct CompletedToggle {
    pub is_active: Signal<bool>,
    pub count: Memo<usize>,
    pub on_toggle: Callback<()>,
}

#[derive(Clone)]
pub struct ActionableToggle {
    pub is_active: Signal<bool>,
    pub count: Memo<usize>,
    pub on_toggle: Callback<()>,
}
