use leptos::prelude::*;
use north_dto::TagInfo;
use north_ui::IconKind;

use super::task_meta_item::{TaskMetaItem, TaskMetaItemVariant};

#[component]
pub fn TaskMetaView(
    #[prop(default = None)] recurrence_label: Option<String>,
    #[prop(default = None)] on_recurrence_click: Option<Callback<()>>,
    #[prop(default = None)] start_at_display: Option<String>,
    #[prop(default = TaskMetaItemVariant::Info)] start_at_variant: TaskMetaItemVariant,
    #[prop(default = 0)] subtask_count: i64,
    #[prop(default = 0)] completed_subtask_count: i64,
    #[prop(default = None)] on_toggle_subtasks: Option<Callback<()>>,
    #[prop(default = None)] due_date_display: Option<String>,
    #[prop(default = TaskMetaItemVariant::Info)] due_date_variant: TaskMetaItemVariant,
    #[prop(default = vec![])] tags: Vec<TagInfo>,
    #[prop(default = false)] show_review: bool,
    #[prop(default = None)] reviewed_at_display: Option<String>,
    #[prop(default = Callback::new(|_| {}))] on_review: Callback<()>,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!(
            "flex items-center gap-2 text-xs text-text-tertiary flex-wrap {class}"
        )>
            {show_review.then(|| {
                view! {
                    <TaskMetaItem
                        icon=IconKind::Review
                        on_click=Some(on_review)
                        variant=TaskMetaItemVariant::Info
                        class="hover:text-accent"
                    >
                        "Mark reviewed"
                    </TaskMetaItem>
                }
            })}
            {reviewed_at_display.map(|display| {
                view! {
                    <TaskMetaItem
                        icon=IconKind::Clock
                        variant=TaskMetaItemVariant::Info
                    >
                        {display}
                    </TaskMetaItem>
                }
            })}
            {recurrence_label.map(|label| {
                view! {
                    <TaskMetaItem
                        icon=IconKind::Recurrence
                        on_click=on_recurrence_click
                        class="text-accent hover:text-accent-hover"
                    >
                        {label}
                    </TaskMetaItem>
                }
            })}
            {start_at_display.map(|display| {
                view! {
                    <TaskMetaItem
                        icon=IconKind::Calendar
                        variant=start_at_variant
                    >
                        {display}
                    </TaskMetaItem>
                }
            })}
            {(subtask_count > 0).then(|| {
                view! {
                    <TaskMetaItem
                        icon=IconKind::Subtask
                        on_click=on_toggle_subtasks
                        variant=TaskMetaItemVariant::Info
                        class="hover:text-accent pl-2"
                    >
                        {format!(
                            "{completed_subtask_count}/{subtask_count} subtask{}",
                            if subtask_count == 1 { "" } else { "s" },
                        )}
                    </TaskMetaItem>
                }
            })}
            {due_date_display.map(|display| {
                view! {
                    <TaskMetaItem
                        icon=IconKind::Clock
                        variant=due_date_variant
                    >
                        {display}
                    </TaskMetaItem>
                }
            })}
            {(!tags.is_empty()).then(|| {
                tags.into_iter()
                    .map(|tag| {
                        view! {
                            <TaskMetaItem
                                icon=IconKind::Tag
                                style=format!("color: {}", tag.color)
                            >
                                {tag.name}
                            </TaskMetaItem>
                        }
                    })
                    .collect::<Vec<_>>()
            })}
        </div>
    }
}
