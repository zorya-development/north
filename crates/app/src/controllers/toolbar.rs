use leptos::prelude::*;

use super::actionable::ActionableController;
use crate::containers::traversable_task_list::{ActionableToggle, CompletedToggle, ToolbarConfig};

pub fn build_toolbar(
    show_add_task: bool,
    completed: Option<(RwSignal<bool>, Memo<usize>)>,
    actionable: &ActionableController,
) -> ToolbarConfig {
    let actionable = *actionable;
    ToolbarConfig {
        enabled: true,
        show_add_task,
        completed: completed.map(|(show_completed, count)| CompletedToggle {
            is_active: show_completed.into(),
            count,
            on_toggle: Callback::new(move |()| {
                show_completed.update(|v| *v = !*v);
            }),
        }),
        actionable: Some(ActionableToggle {
            is_active: actionable.hide,
            count: actionable.count,
            on_toggle: Callback::new(move |()| actionable.toggle()),
        }),
    }
}
