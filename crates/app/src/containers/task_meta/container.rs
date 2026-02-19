use leptos::prelude::*;
use north_dto::{RecurrenceRule, RecurrenceType, TagInfo};

use super::task_meta_item::TaskMetaItemVariant;
use super::view::TaskMetaView;

#[component]
pub fn TaskMeta(
    start_at: Option<chrono::DateTime<chrono::Utc>>,
    due_date: Option<chrono::NaiveDate>,
    tags: Vec<TagInfo>,
    #[prop(default = None)] reviewed_at: Option<chrono::NaiveDate>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = 0)] subtask_count: i64,
    #[prop(default = 0)] completed_subtask_count: i64,
    #[prop(optional)] on_toggle_subtasks: Option<Callback<()>>,
    #[prop(default = Callback::new(|_| {}))] on_review: Callback<()>,
    #[prop(default = None)] recurrence_type: Option<RecurrenceType>,
    #[prop(default = None)] recurrence_rule: Option<String>,
    #[prop(optional)] on_recurrence_click: Option<Callback<()>>,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let has_meta = start_at.is_some()
        || due_date.is_some()
        || !tags.is_empty()
        || show_review
        || subtask_count > 0
        || recurrence_type.is_some();

    has_meta.then(|| {
        let recurrence_label = recurrence_type.and_then(|_| {
            recurrence_rule
                .as_deref()
                .and_then(RecurrenceRule::parse)
                .map(|r| r.summarize())
        });

        let (start_at_display, start_at_variant) = match start_at {
            Some(dt) => {
                let variant = if dt < chrono::Utc::now() {
                    TaskMetaItemVariant::Danger
                } else {
                    TaskMetaItemVariant::Info
                };
                (Some(dt.format("%b %-d, %-I:%M %p").to_string()), variant)
            }
            None => (None, TaskMetaItemVariant::Info),
        };

        let (due_date_display, due_date_variant) = match due_date {
            Some(d) => {
                let variant = if d < chrono::Utc::now().date_naive() {
                    TaskMetaItemVariant::Danger
                } else {
                    TaskMetaItemVariant::Info
                };
                (Some(format!("Due {d}")), variant)
            }
            None => (None, TaskMetaItemVariant::Info),
        };

        let reviewed_at_display = show_review.then(|| match reviewed_at {
            Some(d) => format!("Reviewed {d}"),
            None => "Never reviewed".to_string(),
        });

        view! {
            <TaskMetaView
                recurrence_label=recurrence_label
                on_recurrence_click=on_recurrence_click
                start_at_display=start_at_display
                start_at_variant=start_at_variant
                subtask_count=subtask_count
                completed_subtask_count=completed_subtask_count
                on_toggle_subtasks=on_toggle_subtasks
                due_date_display=due_date_display
                due_date_variant=due_date_variant
                tags=tags
                show_review=show_review
                reviewed_at_display=reviewed_at_display
                on_review=on_review
                class=class
            />
        }
    })
}
