use diesel::dsl::max;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::models::{ColumnChangeset, ColumnRow, NewColumn};
use north_db::schema::{project_columns, projects, tasks};
use north_db::DbPool;
use north_domain::{Column, CreateColumn, UpdateColumn};

use crate::{ServiceError, ServiceResult};

pub struct ColumnService;

impl ColumnService {
    pub async fn get_for_project(pool: &DbPool, project_id: i64) -> ServiceResult<Vec<Column>> {
        let mut conn = pool.get().await?;
        let rows = project_columns::table
            .filter(project_columns::project_id.eq(project_id))
            .order(project_columns::position.asc())
            .select(ColumnRow::as_select())
            .load(&mut conn)
            .await?;
        Ok(rows.into_iter().map(Column::from).collect())
    }

    pub async fn create(
        pool: &DbPool,
        user_id: i64,
        project_id: i64,
        input: &CreateColumn,
    ) -> ServiceResult<Column> {
        let mut conn = pool.get().await?;

        // Verify project ownership
        let exists: bool = diesel::select(diesel::dsl::exists(
            projects::table
                .filter(projects::id.eq(project_id))
                .filter(projects::user_id.eq(user_id)),
        ))
        .get_result(&mut conn)
        .await?;

        if !exists {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        let max_pos: Option<i32> = project_columns::table
            .filter(project_columns::project_id.eq(project_id))
            .select(max(project_columns::position))
            .first(&mut conn)
            .await?;
        let position = max_pos.unwrap_or(-1) + 1;

        let color = input.color.as_deref().unwrap_or("#6b7280");
        let is_done = input.is_done.unwrap_or(false);

        let row = diesel::insert_into(project_columns::table)
            .values(&NewColumn {
                project_id,
                name: &input.name,
                color,
                position,
                is_done,
            })
            .returning(ColumnRow::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(Column::from(row))
    }

    pub async fn update(
        pool: &DbPool,
        user_id: i64,
        column_id: i64,
        input: &UpdateColumn,
    ) -> ServiceResult<Column> {
        let mut conn = pool.get().await?;

        // Verify ownership via project join
        let _existing = project_columns::table
            .inner_join(projects::table.on(projects::id.eq(project_columns::project_id)))
            .filter(project_columns::id.eq(column_id))
            .filter(projects::user_id.eq(user_id))
            .select(ColumnRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Column not found".into()))?;

        let changeset = ColumnChangeset {
            name: input.name.as_deref(),
            color: input.color.as_deref(),
            position: input.position,
            is_done: input.is_done,
        };

        let row = diesel::update(project_columns::table.filter(project_columns::id.eq(column_id)))
            .set(&changeset)
            .returning(ColumnRow::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(Column::from(row))
    }

    pub async fn delete(pool: &DbPool, user_id: i64, column_id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;

        // Verify ownership
        let existing = project_columns::table
            .inner_join(projects::table.on(projects::id.eq(project_columns::project_id)))
            .filter(project_columns::id.eq(column_id))
            .filter(projects::user_id.eq(user_id))
            .select(ColumnRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Column not found".into()))?;

        // Check if tasks use this column
        let task_count: i64 = tasks::table
            .filter(tasks::column_id.eq(column_id))
            .count()
            .get_result(&mut conn)
            .await?;

        if task_count > 0 {
            // Find first alternative column
            let first_column: Option<i64> = project_columns::table
                .filter(project_columns::project_id.eq(existing.project_id))
                .filter(project_columns::id.ne(column_id))
                .order(project_columns::position.asc())
                .select(project_columns::id)
                .first(&mut conn)
                .await
                .optional()?;

            match first_column {
                Some(new_col_id) => {
                    diesel::update(tasks::table.filter(tasks::column_id.eq(column_id)))
                        .set(tasks::column_id.eq(new_col_id))
                        .execute(&mut conn)
                        .await?;
                }
                None => {
                    return Err(ServiceError::BadRequest(
                        "Cannot delete the only column while tasks are assigned to it".into(),
                    ));
                }
            }
        }

        diesel::delete(project_columns::table.filter(project_columns::id.eq(column_id)))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn get_all_for_user(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<Column>> {
        let mut conn = pool.get().await?;
        let rows = project_columns::table
            .inner_join(projects::table.on(projects::id.eq(project_columns::project_id)))
            .filter(projects::user_id.eq(user_id))
            .filter(projects::archived.eq(false))
            .order(project_columns::position.asc())
            .select(ColumnRow::as_select())
            .load(&mut conn)
            .await?;
        Ok(rows.into_iter().map(Column::from).collect())
    }

    /// Check if a column is marked as done
    pub async fn is_done(pool: &DbPool, column_id: i64) -> ServiceResult<Option<bool>> {
        let mut conn = pool.get().await?;
        let result = project_columns::table
            .filter(project_columns::id.eq(column_id))
            .select(project_columns::is_done)
            .first(&mut conn)
            .await
            .optional()?;
        Ok(result)
    }
}
