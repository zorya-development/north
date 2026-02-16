use std::collections::HashSet;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::schema::{projects, tags, task_tags, tasks};
use north_db::DbPool;
use north_domain::{Condition, FilterExpr, FilterField, FilterOp, FilterValue};

use crate::ServiceResult;

/// Evaluates a FilterExpr into a set of matching task IDs for the given user.
pub fn eval_expr<'a>(
    pool: &'a DbPool,
    user_id: i64,
    expr: &'a FilterExpr,
) -> futures_util::future::BoxFuture<'a, ServiceResult<HashSet<i64>>> {
    Box::pin(async move {
        match expr {
            FilterExpr::Condition(cond) => eval_condition(pool, user_id, cond).await,
            FilterExpr::And(a, b) => {
                let set_a = eval_expr(pool, user_id, a).await?;
                let set_b = eval_expr(pool, user_id, b).await?;
                Ok(set_a.intersection(&set_b).copied().collect())
            }
            FilterExpr::Or(a, b) => {
                let set_a = eval_expr(pool, user_id, a).await?;
                let set_b = eval_expr(pool, user_id, b).await?;
                Ok(set_a.union(&set_b).copied().collect())
            }
            FilterExpr::Not(e) => {
                let set_e = eval_expr(pool, user_id, e).await?;
                let all = all_user_task_ids(pool, user_id).await?;
                Ok(all.difference(&set_e).copied().collect())
            }
        }
    })
}

async fn all_user_task_ids(pool: &DbPool, user_id: i64) -> ServiceResult<HashSet<i64>> {
    let mut conn = pool.get().await?;
    let ids: Vec<i64> = tasks::table
        .filter(tasks::user_id.eq(user_id))
        .filter(tasks::parent_id.is_null())
        .select(tasks::id)
        .load(&mut conn)
        .await?;
    Ok(ids.into_iter().collect())
}

async fn eval_condition(
    pool: &DbPool,
    user_id: i64,
    cond: &Condition,
) -> ServiceResult<HashSet<i64>> {
    match cond.field {
        FilterField::Title => eval_text_field(pool, user_id, cond, TextField::Title).await,
        FilterField::Body => eval_text_field(pool, user_id, cond, TextField::Body).await,
        FilterField::Status => eval_status(pool, user_id, cond).await,
        FilterField::Project => eval_project(pool, user_id, cond).await,
        FilterField::Tags => eval_tags(pool, user_id, cond).await,
        FilterField::DueDate => eval_date_field(pool, user_id, cond, DateField::DueDate).await,
        FilterField::StartAt => eval_date_field(pool, user_id, cond, DateField::StartAt).await,
        FilterField::Created => eval_date_field(pool, user_id, cond, DateField::Created).await,
        FilterField::Updated => eval_date_field(pool, user_id, cond, DateField::Updated).await,
    }
}

fn glob_to_sql_like(glob: &str) -> String {
    glob.replace('%', "\\%")
        .replace('_', "\\_")
        .replace('*', "%")
        .replace('?', "_")
}

fn value_as_str(v: &FilterValue) -> Option<&str> {
    match v {
        FilterValue::String(s) => Some(s),
        _ => None,
    }
}

fn value_as_strings(v: &FilterValue) -> Vec<String> {
    match v {
        FilterValue::Array(items) => items
            .iter()
            .filter_map(|i| match i {
                FilterValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        FilterValue::String(s) => vec![s.clone()],
        _ => vec![],
    }
}

enum TextField {
    Title,
    Body,
}

async fn eval_text_field(
    pool: &DbPool,
    user_id: i64,
    cond: &Condition,
    field: TextField,
) -> ServiceResult<HashSet<i64>> {
    let mut conn = pool.get().await?;
    let s = value_as_str(&cond.value).unwrap_or("");

    let mut query = tasks::table
        .filter(tasks::user_id.eq(user_id))
        .filter(tasks::parent_id.is_null())
        .into_boxed();

    match (&field, &cond.op) {
        (TextField::Title, FilterOp::Eq) => {
            query = query.filter(tasks::title.ilike(s));
        }
        (TextField::Title, FilterOp::Ne) => {
            query = query.filter(tasks::title.not_ilike(s));
        }
        (TextField::Title, FilterOp::GlobMatch) => {
            let pattern = glob_to_sql_like(s);
            query = query.filter(tasks::title.ilike(pattern));
        }
        (TextField::Title, FilterOp::GlobNotMatch) => {
            let pattern = glob_to_sql_like(s);
            query = query.filter(tasks::title.not_ilike(pattern));
        }
        (TextField::Body, FilterOp::Eq) => {
            query = query.filter(tasks::body.ilike(s));
        }
        (TextField::Body, FilterOp::Ne) => {
            query = query.filter(tasks::body.not_ilike(s));
        }
        (TextField::Body, FilterOp::GlobMatch) => {
            let pattern = glob_to_sql_like(s);
            query = query.filter(tasks::body.ilike(pattern));
        }
        (TextField::Body, FilterOp::GlobNotMatch) => {
            let pattern = glob_to_sql_like(s);
            query = query.filter(tasks::body.not_ilike(pattern));
        }
        (TextField::Title, FilterOp::Is) if cond.value == FilterValue::Null => {
            return Ok(HashSet::new());
        }
        (TextField::Body, FilterOp::Is) if cond.value == FilterValue::Null => {
            query = query.filter(tasks::body.is_null());
        }
        (TextField::Body, FilterOp::IsNot) if cond.value == FilterValue::Null => {
            query = query.filter(tasks::body.is_not_null());
        }
        _ => return Ok(HashSet::new()),
    }

    let ids: Vec<i64> = query.select(tasks::id).load(&mut conn).await?;
    Ok(ids.into_iter().collect())
}

async fn eval_status(pool: &DbPool, user_id: i64, cond: &Condition) -> ServiceResult<HashSet<i64>> {
    let mut conn = pool.get().await?;
    let s = value_as_str(&cond.value).unwrap_or("").to_uppercase();

    let mut query = tasks::table
        .filter(tasks::user_id.eq(user_id))
        .filter(tasks::parent_id.is_null())
        .into_boxed();

    let is_completed_check = matches!(s.as_str(), "COMPLETED" | "DONE");

    match cond.op {
        FilterOp::Eq => {
            if is_completed_check {
                query = query.filter(tasks::completed_at.is_not_null());
            } else {
                query = query.filter(tasks::completed_at.is_null());
            }
        }
        FilterOp::Ne => {
            if is_completed_check {
                query = query.filter(tasks::completed_at.is_null());
            } else {
                query = query.filter(tasks::completed_at.is_not_null());
            }
        }
        FilterOp::In => {
            let values = value_as_strings(&cond.value);
            let has_completed = values
                .iter()
                .any(|v| matches!(v.to_uppercase().as_str(), "COMPLETED" | "DONE"));
            let has_active = values
                .iter()
                .any(|v| matches!(v.to_uppercase().as_str(), "ACTIVE" | "OPEN"));
            if has_completed && has_active {
                // Both — no filter needed, return all
            } else if has_completed {
                query = query.filter(tasks::completed_at.is_not_null());
            } else if has_active {
                query = query.filter(tasks::completed_at.is_null());
            } else {
                return Ok(HashSet::new());
            }
        }
        _ => return Ok(HashSet::new()),
    }

    let ids: Vec<i64> = query.select(tasks::id).load(&mut conn).await?;
    Ok(ids.into_iter().collect())
}

async fn eval_project(
    pool: &DbPool,
    user_id: i64,
    cond: &Condition,
) -> ServiceResult<HashSet<i64>> {
    let mut conn = pool.get().await?;

    match cond.op {
        FilterOp::Is if cond.value == FilterValue::Null => {
            let ids: Vec<i64> = tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .filter(tasks::project_id.is_null())
                .select(tasks::id)
                .load(&mut conn)
                .await?;
            return Ok(ids.into_iter().collect());
        }
        FilterOp::IsNot if cond.value == FilterValue::Null => {
            let ids: Vec<i64> = tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .filter(tasks::project_id.is_not_null())
                .select(tasks::id)
                .load(&mut conn)
                .await?;
            return Ok(ids.into_iter().collect());
        }
        _ => {}
    }

    let project_name = value_as_str(&cond.value).unwrap_or("");

    let project_ids: Vec<i64> = match cond.op {
        FilterOp::Eq => {
            projects::table
                .filter(projects::user_id.eq(user_id))
                .filter(projects::title.ilike(project_name))
                .select(projects::id)
                .load(&mut conn)
                .await?
        }
        FilterOp::Ne => {
            let matching_pids: Vec<i64> = projects::table
                .filter(projects::user_id.eq(user_id))
                .filter(projects::title.ilike(project_name))
                .select(projects::id)
                .load(&mut conn)
                .await?;

            let ids: Vec<i64> = tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .filter(
                    tasks::project_id
                        .is_null()
                        .or(tasks::project_id.ne_all(&matching_pids)),
                )
                .select(tasks::id)
                .load(&mut conn)
                .await?;
            return Ok(ids.into_iter().collect());
        }
        FilterOp::GlobMatch => {
            let pattern = glob_to_sql_like(project_name);
            projects::table
                .filter(projects::user_id.eq(user_id))
                .filter(projects::title.ilike(pattern))
                .select(projects::id)
                .load(&mut conn)
                .await?
        }
        FilterOp::GlobNotMatch => {
            let pattern = glob_to_sql_like(project_name);
            let matching_pids: Vec<i64> = projects::table
                .filter(projects::user_id.eq(user_id))
                .filter(projects::title.ilike(pattern))
                .select(projects::id)
                .load(&mut conn)
                .await?;

            let ids: Vec<i64> = tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .filter(
                    tasks::project_id
                        .is_null()
                        .or(tasks::project_id.ne_all(&matching_pids)),
                )
                .select(tasks::id)
                .load(&mut conn)
                .await?;
            return Ok(ids.into_iter().collect());
        }
        FilterOp::In => {
            let names = value_as_strings(&cond.value);
            projects::table
                .filter(projects::user_id.eq(user_id))
                .filter(projects::title.eq_any(&names))
                .select(projects::id)
                .load(&mut conn)
                .await?
        }
        _ => return Ok(HashSet::new()),
    };

    let ids: Vec<i64> = tasks::table
        .filter(tasks::user_id.eq(user_id))
        .filter(tasks::parent_id.is_null())
        .filter(tasks::project_id.eq_any(&project_ids))
        .select(tasks::id)
        .load(&mut conn)
        .await?;
    Ok(ids.into_iter().collect())
}

async fn eval_tags(pool: &DbPool, user_id: i64, cond: &Condition) -> ServiceResult<HashSet<i64>> {
    let mut conn = pool.get().await?;

    let tag_ids: Vec<i64> = match cond.op {
        FilterOp::Eq => {
            let name = value_as_str(&cond.value).unwrap_or("");
            tags::table
                .filter(tags::user_id.eq(user_id))
                .filter(tags::name.eq(name))
                .select(tags::id)
                .load(&mut conn)
                .await?
        }
        FilterOp::Ne => {
            let name = value_as_str(&cond.value).unwrap_or("");
            let matching_tag_ids: Vec<i64> = tags::table
                .filter(tags::user_id.eq(user_id))
                .filter(tags::name.eq(name))
                .select(tags::id)
                .load(&mut conn)
                .await?;

            let tagged_task_ids: Vec<i64> = task_tags::table
                .filter(task_tags::tag_id.eq_any(&matching_tag_ids))
                .select(task_tags::task_id)
                .load(&mut conn)
                .await?;
            let tagged_set: HashSet<i64> = tagged_task_ids.into_iter().collect();

            let all = all_user_task_ids(pool, user_id).await?;
            return Ok(all.difference(&tagged_set).copied().collect());
        }
        FilterOp::GlobMatch => {
            let pattern = glob_to_sql_like(value_as_str(&cond.value).unwrap_or(""));
            tags::table
                .filter(tags::user_id.eq(user_id))
                .filter(tags::name.ilike(pattern))
                .select(tags::id)
                .load(&mut conn)
                .await?
        }
        FilterOp::GlobNotMatch => {
            let pattern = glob_to_sql_like(value_as_str(&cond.value).unwrap_or(""));
            let matching_tag_ids: Vec<i64> = tags::table
                .filter(tags::user_id.eq(user_id))
                .filter(tags::name.ilike(pattern))
                .select(tags::id)
                .load(&mut conn)
                .await?;

            let tagged_task_ids: Vec<i64> = task_tags::table
                .filter(task_tags::tag_id.eq_any(&matching_tag_ids))
                .select(task_tags::task_id)
                .load(&mut conn)
                .await?;
            let tagged_set: HashSet<i64> = tagged_task_ids.into_iter().collect();

            let all = all_user_task_ids(pool, user_id).await?;
            return Ok(all.difference(&tagged_set).copied().collect());
        }
        FilterOp::In => {
            let names = value_as_strings(&cond.value);
            tags::table
                .filter(tags::user_id.eq(user_id))
                .filter(tags::name.eq_any(&names))
                .select(tags::id)
                .load(&mut conn)
                .await?
        }
        FilterOp::NotIn => {
            let names = value_as_strings(&cond.value);
            let matching_tag_ids: Vec<i64> = tags::table
                .filter(tags::user_id.eq(user_id))
                .filter(tags::name.eq_any(&names))
                .select(tags::id)
                .load(&mut conn)
                .await?;

            let tagged_task_ids: Vec<i64> = task_tags::table
                .filter(task_tags::tag_id.eq_any(&matching_tag_ids))
                .select(task_tags::task_id)
                .load(&mut conn)
                .await?;
            let tagged_set: HashSet<i64> = tagged_task_ids.into_iter().collect();

            let all = all_user_task_ids(pool, user_id).await?;
            return Ok(all.difference(&tagged_set).copied().collect());
        }
        _ => return Ok(HashSet::new()),
    };

    let task_ids: Vec<i64> = task_tags::table
        .filter(task_tags::tag_id.eq_any(&tag_ids))
        .select(task_tags::task_id)
        .load(&mut conn)
        .await?;

    let all = all_user_task_ids(pool, user_id).await?;
    let tagged: HashSet<i64> = task_ids.into_iter().collect();
    Ok(all.intersection(&tagged).copied().collect())
}

enum DateField {
    DueDate,
    StartAt,
    Created,
    Updated,
}

async fn eval_date_field(
    pool: &DbPool,
    user_id: i64,
    cond: &Condition,
    field: DateField,
) -> ServiceResult<HashSet<i64>> {
    let mut conn = pool.get().await?;

    if cond.value == FilterValue::Null {
        let mut query = tasks::table
            .filter(tasks::user_id.eq(user_id))
            .filter(tasks::parent_id.is_null())
            .into_boxed();

        match (&field, &cond.op) {
            (DateField::DueDate, FilterOp::Is) => {
                query = query.filter(tasks::due_date.is_null());
            }
            (DateField::DueDate, FilterOp::IsNot) => {
                query = query.filter(tasks::due_date.is_not_null());
            }
            (DateField::StartAt, FilterOp::Is) => {
                query = query.filter(tasks::start_at.is_null());
            }
            (DateField::StartAt, FilterOp::IsNot) => {
                query = query.filter(tasks::start_at.is_not_null());
            }
            _ => return Ok(HashSet::new()),
        }

        let ids: Vec<i64> = query.select(tasks::id).load(&mut conn).await?;
        return Ok(ids.into_iter().collect());
    }

    let date_str = value_as_str(&cond.value).unwrap_or("");

    match field {
        DateField::DueDate => {
            let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
                crate::ServiceError::BadRequest(format!("Invalid date format: {date_str}"))
            })?;

            let mut query = tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .into_boxed();

            match cond.op {
                FilterOp::Eq => query = query.filter(tasks::due_date.eq(date)),
                FilterOp::Ne => {
                    query = query.filter(tasks::due_date.ne(date).or(tasks::due_date.is_null()))
                }
                FilterOp::Gt => query = query.filter(tasks::due_date.gt(date)),
                FilterOp::Lt => query = query.filter(tasks::due_date.lt(date)),
                FilterOp::Gte => query = query.filter(tasks::due_date.ge(date)),
                FilterOp::Lte => query = query.filter(tasks::due_date.le(date)),
                _ => return Ok(HashSet::new()),
            }

            let ids: Vec<i64> = query.select(tasks::id).load(&mut conn).await?;
            Ok(ids.into_iter().collect())
        }
        DateField::StartAt => {
            let dt = parse_datetime(date_str).map_err(|_| {
                crate::ServiceError::BadRequest(format!("Invalid datetime format: {date_str}"))
            })?;

            let mut query = tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .into_boxed();

            match cond.op {
                FilterOp::Eq => query = query.filter(tasks::start_at.eq(dt)),
                FilterOp::Ne => {
                    query = query.filter(tasks::start_at.ne(dt).or(tasks::start_at.is_null()))
                }
                FilterOp::Gt => query = query.filter(tasks::start_at.gt(dt)),
                FilterOp::Lt => query = query.filter(tasks::start_at.lt(dt)),
                FilterOp::Gte => query = query.filter(tasks::start_at.ge(dt)),
                FilterOp::Lte => query = query.filter(tasks::start_at.le(dt)),
                _ => return Ok(HashSet::new()),
            }

            let ids: Vec<i64> = query.select(tasks::id).load(&mut conn).await?;
            Ok(ids.into_iter().collect())
        }
        DateField::Created => {
            let dt = parse_datetime(date_str).map_err(|_| {
                crate::ServiceError::BadRequest(format!("Invalid datetime format: {date_str}"))
            })?;

            let mut query = tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .into_boxed();

            match cond.op {
                FilterOp::Gt => query = query.filter(tasks::created_at.gt(dt)),
                FilterOp::Lt => query = query.filter(tasks::created_at.lt(dt)),
                FilterOp::Gte => query = query.filter(tasks::created_at.ge(dt)),
                FilterOp::Lte => query = query.filter(tasks::created_at.le(dt)),
                _ => return Ok(HashSet::new()),
            }

            let ids: Vec<i64> = query.select(tasks::id).load(&mut conn).await?;
            Ok(ids.into_iter().collect())
        }
        DateField::Updated => {
            let dt = parse_datetime(date_str).map_err(|_| {
                crate::ServiceError::BadRequest(format!("Invalid datetime format: {date_str}"))
            })?;

            let mut query = tasks::table
                .filter(tasks::user_id.eq(user_id))
                .filter(tasks::parent_id.is_null())
                .into_boxed();

            match cond.op {
                FilterOp::Gt => query = query.filter(tasks::updated_at.gt(dt)),
                FilterOp::Lt => query = query.filter(tasks::updated_at.lt(dt)),
                FilterOp::Gte => query = query.filter(tasks::updated_at.ge(dt)),
                FilterOp::Lte => query = query.filter(tasks::updated_at.le(dt)),
                _ => return Ok(HashSet::new()),
            }

            let ids: Vec<i64> = query.select(tasks::id).load(&mut conn).await?;
            Ok(ids.into_iter().collect())
        }
    }
}

fn parse_datetime(s: &str) -> Result<chrono::DateTime<chrono::Utc>, chrono::ParseError> {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        return Ok(dt);
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt.and_utc());
    }
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").map(|dt| dt.and_utc())
}
