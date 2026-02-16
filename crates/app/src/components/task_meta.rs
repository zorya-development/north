use leptos::prelude::*;
use north_dto::TagInfo;
use north_ui::{Icon, IconKind};

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
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    let has_meta = start_at.is_some()
        || due_date.is_some()
        || !tags.is_empty()
        || (show_review && reviewed_at.is_some())
        || subtask_count > 0;

    has_meta.then(|| {
        view! {
            <div class=format!(
                "flex items-center gap-2 text-xs text-text-tertiary flex-wrap {class}"
            )>
                {start_at.map(|dt| {
                    let is_overdue = dt < chrono::Utc::now();
                    let class = if is_overdue {
                        "inline-flex items-center gap-1 text-danger"
                    } else {
                        "inline-flex items-center gap-1 text-text-secondary"
                    };
                    let display = dt.format("%b %-d, %-I:%M %p").to_string();
                    view! {
                        <span class=class>
                            <Icon
                                kind=IconKind::Calendar
                                class="w-3 h-3"
                            />
                            {display}
                        </span>
                    }
                })}
                {(subtask_count > 0).then(|| {
                    view! {
                        <span class="inline-flex items-center gap-0.5 \
                                     text-text-secondary \
                                     hover:text-accent cursor-pointer \
                                     transition-colors pl-2"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                if let Some(cb) = on_toggle_subtasks {
                                    cb.run(());
                                }
                            }
                        >
                            <Icon
                                kind=IconKind::Subtask
                                class="w-3 h-3"
                            />
                            {format!(
                                "{completed_subtask_count}/{subtask_count} subtask{}",
                                if subtask_count == 1 { "" } else { "s" },
                            )}
                        </span>
                    }
                })}
                {due_date.map(|d| {
                    let is_overdue = d < chrono::Utc::now().date_naive();
                    let class = if is_overdue {
                        "text-danger"
                    } else {
                        "text-text-secondary"
                    };
                    view! {
                        <span class=class>{format!("Due {d}")}</span>
                    }
                })}
                {(!tags.is_empty()).then(|| {
                    view! {
                        <div class="flex items-center gap-1 flex-wrap">
                            {tags
                                .into_iter()
                                .map(|tag| {
                                    view! {
                                        <span
                                            class="inline-flex items-center \
                                                   gap-0.5 text-xs"
                                            style=format!(
                                                "color: {}",
                                                tag.color,
                                            )
                                        >
                                            <Icon
                                                kind=IconKind::Tag
                                                class="w-3 h-3"
                                            />
                                            {tag.name}
                                        </span>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }
                })}
                {if show_review {
                    Some(view! {
                        <span class="ml-auto whitespace-nowrap">
                            {match reviewed_at {
                                Some(d) => format!("Reviewed {d}"),
                                None => "Never reviewed".to_string(),
                            }}
                        </span>
                    })
                } else {
                    None
                }}
            </div>
        }
    })
}
