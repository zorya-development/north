use diesel::prelude::*;
use diesel::dsl::max;
use diesel_async::RunQueryDsl;
use north_db::models::{ColumnRow, NewColumn, NewProject, ProjectChangeset, ProjectRow};
use north_db::schema::{project_columns, projects, users};
use north_db::sql_types::ProjectViewTypeMapping;
use north_db::DbPool;
use north_domain::{
    Column, CreateProject, Project, ProjectViewType, ProjectWithColumns, UpdateProject,
    UserSettings,
};

use crate::{ServiceError, ServiceResult};

pub struct ProjectService;

impl ProjectService {
    pub async fn get_active(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<Project>> {
        let mut conn = pool.get().await?;
        let rows = projects::table
            .filter(projects::user_id.eq(user_id))
            .filter(projects::archived.eq(false))
            .order((projects::position.asc(), projects::created_at.asc()))
            .select(ProjectRow::as_select())
            .load(&mut conn)
            .await?;
        Ok(rows.into_iter().map(Project::from).collect())
    }

    pub async fn get_archived(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<Project>> {
        let mut conn = pool.get().await?;
        let rows = projects::table
            .filter(projects::user_id.eq(user_id))
            .filter(projects::archived.eq(true))
            .order(projects::updated_at.desc())
            .select(ProjectRow::as_select())
            .load(&mut conn)
            .await?;
        Ok(rows.into_iter().map(Project::from).collect())
    }

    pub async fn get_by_id(
        pool: &DbPool,
        user_id: i64,
        id: i64,
    ) -> ServiceResult<Project> {
        let mut conn = pool.get().await?;
        let row = projects::table
            .filter(projects::id.eq(id))
            .filter(projects::user_id.eq(user_id))
            .select(ProjectRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Project not found".into()))?;
        Ok(Project::from(row))
    }

    pub async fn get_with_columns(
        pool: &DbPool,
        user_id: i64,
        id: i64,
    ) -> ServiceResult<ProjectWithColumns> {
        let mut conn = pool.get().await?;

        let proj_row = projects::table
            .filter(projects::id.eq(id))
            .filter(projects::user_id.eq(user_id))
            .select(ProjectRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Project not found".into()))?;

        let col_rows = project_columns::table
            .filter(project_columns::project_id.eq(id))
            .order(project_columns::position.asc())
            .select(ColumnRow::as_select())
            .load(&mut conn)
            .await?;

        Ok(ProjectWithColumns {
            project: Project::from(proj_row),
            columns: col_rows.into_iter().map(Column::from).collect(),
        })
    }

    pub async fn list_with_columns(
        pool: &DbPool,
        user_id: i64,
    ) -> ServiceResult<Vec<ProjectWithColumns>> {
        let mut conn = pool.get().await?;

        let proj_rows = projects::table
            .filter(projects::user_id.eq(user_id))
            .filter(projects::archived.eq(false))
            .order(projects::position.asc())
            .select(ProjectRow::as_select())
            .load(&mut conn)
            .await?;

        let proj_ids: Vec<i64> = proj_rows.iter().map(|p| p.id).collect();

        let col_rows = project_columns::table
            .filter(project_columns::project_id.eq_any(&proj_ids))
            .order(project_columns::position.asc())
            .select(ColumnRow::as_select())
            .load(&mut conn)
            .await?;

        let mut cols_map: std::collections::HashMap<i64, Vec<Column>> =
            std::collections::HashMap::new();
        for col in col_rows {
            cols_map
                .entry(col.project_id)
                .or_default()
                .push(Column::from(col));
        }

        Ok(proj_rows
            .into_iter()
            .map(|row| {
                let id = row.id;
                ProjectWithColumns {
                    project: Project::from(row),
                    columns: cols_map.remove(&id).unwrap_or_default(),
                }
            })
            .collect())
    }

    pub async fn create(
        pool: &DbPool,
        user_id: i64,
        input: &CreateProject,
    ) -> ServiceResult<ProjectWithColumns> {
        let mut conn = pool.get().await?;

        let max_pos: Option<i32> = projects::table
            .filter(projects::user_id.eq(user_id))
            .select(max(projects::position))
            .first(&mut conn)
            .await?;
        let position = max_pos.unwrap_or(-1) + 1;

        let vt = input
            .view_type
            .as_ref()
            .unwrap_or(&ProjectViewType::List);

        let proj_row = diesel::insert_into(projects::table)
            .values(&NewProject {
                user_id,
                title: &input.title,
                description: input.description.as_deref(),
                view_type: ProjectViewTypeMapping::from(vt.clone()),
                position,
            })
            .returning(ProjectRow::as_returning())
            .get_result(&mut conn)
            .await?;

        // Fetch user settings for default columns
        let settings_val: serde_json::Value = users::table
            .filter(users::id.eq(user_id))
            .select(users::settings)
            .first(&mut conn)
            .await?;
        let settings: UserSettings = serde_json::from_value(settings_val).unwrap_or_default();

        let mut columns = Vec::new();
        for (i, default_col) in settings.default_columns.iter().enumerate() {
            let col_row = diesel::insert_into(project_columns::table)
                .values(&NewColumn {
                    project_id: proj_row.id,
                    name: &default_col.name,
                    color: &default_col.color,
                    position: i as i32,
                    is_done: default_col.is_done,
                })
                .returning(ColumnRow::as_returning())
                .get_result(&mut conn)
                .await?;
            columns.push(Column::from(col_row));
        }

        Ok(ProjectWithColumns {
            project: Project::from(proj_row),
            columns,
        })
    }

    pub async fn update(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        input: &UpdateProject,
    ) -> ServiceResult<Project> {
        let mut conn = pool.get().await?;

        // Verify ownership
        let existing = projects::table
            .filter(projects::id.eq(id))
            .filter(projects::user_id.eq(user_id))
            .select(ProjectRow::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| ServiceError::NotFound("Project not found".into()))?;

        let changeset = ProjectChangeset {
            title: input.title.as_deref(),
            description: input.description.as_ref().map(|d| Some(d.as_str())),
            view_type: input.view_type.as_ref().map(|vt| {
                ProjectViewTypeMapping::from(vt.clone())
            }),
            position: input.position,
            color: input.color.as_deref(),
            archived: input.archived,
        };

        // Only update if there's something to change
        let has_changes = changeset.title.is_some()
            || changeset.description.is_some()
            || changeset.view_type.is_some()
            || changeset.position.is_some()
            || changeset.color.is_some()
            || changeset.archived.is_some();

        if has_changes {
            let row = diesel::update(projects::table.filter(projects::id.eq(id)))
                .set(&changeset)
                .returning(ProjectRow::as_returning())
                .get_result(&mut conn)
                .await?;
            Ok(Project::from(row))
        } else {
            Ok(Project::from(existing))
        }
    }

    pub async fn update_details(
        pool: &DbPool,
        user_id: i64,
        id: i64,
        title: &str,
        color: &str,
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            projects::table
                .filter(projects::id.eq(id))
                .filter(projects::user_id.eq(user_id)),
        )
        .set((projects::title.eq(title), projects::color.eq(color)))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Project not found".into()));
        }
        Ok(())
    }

    pub async fn archive(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            projects::table
                .filter(projects::id.eq(id))
                .filter(projects::user_id.eq(user_id)),
        )
        .set(projects::archived.eq(true))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Project not found".into()));
        }
        Ok(())
    }

    pub async fn unarchive(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::update(
            projects::table
                .filter(projects::id.eq(id))
                .filter(projects::user_id.eq(user_id)),
        )
        .set(projects::archived.eq(false))
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Project not found".into()));
        }
        Ok(())
    }

    pub async fn delete(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        let affected = diesel::delete(
            projects::table
                .filter(projects::id.eq(id))
                .filter(projects::user_id.eq(user_id)),
        )
        .execute(&mut conn)
        .await?;
        if affected == 0 {
            return Err(ServiceError::NotFound("Project not found".into()));
        }
        Ok(())
    }

    /// Find project by title (case-insensitive) for @project token parsing
    pub async fn find_by_title(
        pool: &DbPool,
        user_id: i64,
        title: &str,
    ) -> ServiceResult<Option<i64>> {
        use diesel::dsl::sql;
        use diesel::sql_types::Text;

        let mut conn = pool.get().await?;
        let id: Option<i64> = projects::table
            .filter(projects::user_id.eq(user_id))
            .filter(projects::archived.eq(false))
            .filter(sql::<diesel::sql_types::Bool>("lower(title) = lower(").bind::<Text, _>(title).sql(")"))
            .select(projects::id)
            .first(&mut conn)
            .await
            .optional()?;
        Ok(id)
    }
}
