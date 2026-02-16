use chrono::Utc;
use leptos::prelude::*;

use super::view::DateTimePickerView;

#[component]
pub fn DateTimePicker(
    task_id: i64,
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    on_set_start_at: Callback<(i64, String)>,
    on_clear_start_at: Callback<i64>,
    #[prop(default = false)] icon_only: bool,
    #[prop(default = false)] always_visible: bool,
) -> impl IntoView {
    let has_start_at = start_at.is_some();
    let is_overdue = start_at.map(|dt| dt < Utc::now()).unwrap_or(false);
    let (popover_open, set_popover_open) = signal(false);

    let picked_date = RwSignal::new(String::new());
    let picked_time = RwSignal::new("09:00".to_string());

    let start_at_display = start_at.map(|dt| dt.format("%b %-d, %-I:%M %p").to_string());

    let initial_date = start_at
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let initial_time = start_at
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|| "09:00".to_string());

    view! {
        <DateTimePickerView
            task_id=task_id
            has_start_at=has_start_at
            is_overdue=is_overdue
            start_at_display=start_at_display
            initial_date=initial_date
            initial_time=initial_time
            popover_open=popover_open
            set_popover_open=set_popover_open
            picked_date=picked_date
            picked_time=picked_time
            on_set_start_at=on_set_start_at
            on_clear_start_at=on_clear_start_at
            icon_only=icon_only
            always_visible=always_visible
        />
    }
}
