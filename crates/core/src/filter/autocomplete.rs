use north_db::DbPool;
use north_dto::DslSuggestion;

use crate::filter::context::{detect_completion_context, DslCompletionContext};
use crate::filter::dsl::FilterField;
use crate::filter::field_registry::TaskFieldRegistry;
use crate::{ProjectService, ServiceResult, TagService};

pub async fn get_dsl_suggestions(
    pool: &DbPool,
    user_id: i64,
    query: &str,
    cursor: usize,
) -> ServiceResult<Vec<DslSuggestion>> {
    let ctx = detect_completion_context(query, cursor);

    match ctx {
        DslCompletionContext::FieldName { partial, start } => {
            let lower = partial.to_lowercase();
            Ok(TaskFieldRegistry::field_names()
                .iter()
                .filter(|name| name.starts_with(&lower))
                .map(|name| DslSuggestion {
                    label: name.to_string(),
                    value: name.to_string(),
                    color: String::new(),
                    start,
                })
                .collect())
        }

        DslCompletionContext::FieldValue {
            field,
            partial,
            start,
        }
        | DslCompletionContext::ArrayValue {
            field,
            partial,
            start,
        } => {
            let lower = partial.to_lowercase();
            match field {
                FilterField::Tags => {
                    let tags = TagService::list(pool, user_id).await?;
                    Ok(tags
                        .into_iter()
                        .filter(|t| t.name.to_lowercase().starts_with(&lower))
                        .map(|t| {
                            let value = format!("'{}'", t.name);
                            DslSuggestion {
                                label: t.name,
                                value,
                                color: t.color,
                                start,
                            }
                        })
                        .collect())
                }
                FilterField::Project => {
                    let filter = north_dto::ProjectFilter {
                        status: Some(north_dto::ProjectStatus::Active),
                    };
                    let projects = ProjectService::list(pool, user_id, &filter).await?;
                    Ok(projects
                        .into_iter()
                        .filter(|p| p.title.to_lowercase().starts_with(&lower))
                        .map(|p| {
                            let value = format!("'{}'", p.title);
                            DslSuggestion {
                                label: p.title,
                                value,
                                color: p.color,
                                start,
                            }
                        })
                        .collect())
                }
                FilterField::Status => {
                    let statuses = ["ACTIVE", "OPEN", "COMPLETED", "DONE"];
                    Ok(statuses
                        .iter()
                        .filter(|s| s.to_lowercase().starts_with(&lower))
                        .map(|s| DslSuggestion {
                            label: s.to_string(),
                            value: s.to_string(),
                            color: String::new(),
                            start,
                        })
                        .collect())
                }
                _ => Ok(vec![]),
            }
        }

        DslCompletionContext::Keyword { partial, start } => {
            let lower = partial.to_lowercase();
            let keywords = ["AND", "OR", "NOT", "ORDER BY"];
            Ok(keywords
                .iter()
                .filter(|kw| kw.to_lowercase().starts_with(&lower))
                .map(|kw| DslSuggestion {
                    label: kw.to_string(),
                    value: kw.to_string(),
                    color: String::new(),
                    start,
                })
                .collect())
        }

        DslCompletionContext::None => Ok(vec![]),
    }
}
