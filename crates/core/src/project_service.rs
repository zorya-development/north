use diesel::dsl::max;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use north_db::models::{NewProject, ProjectChangeset, ProjectRow};
use north_db::schema::projects;
use north_db::sql_types::{ProjectStatusMapping, ProjectViewTypeMapping};
use north_db::DbPool;
use north_dto::{CreateProject, Project, ProjectFilter, ProjectViewType, UpdateProject};

use crate::{ServiceError, ServiceResult};

pub struct ProjectService;

impl ProjectService {
    pub async fn list(
        pool: &DbPool,
        user_id: i64,
        filter: &ProjectFilter,
    ) -> ServiceResult<Vec<Project>> {
        let mut conn = pool.get().await?;
        let mut query = projects::table
            .filter(projects::user_id.eq(user_id))
            .select(ProjectRow::as_select())
            .into_boxed();
        if let Some(ref status) = filter.status {
            query = query.filter(projects::status.eq(ProjectStatusMapping::from(status.clone())));
        }
        query = query.order((projects::position.asc(), projects::created_at.asc()));
        let rows = query.load(&mut conn).await?;
        Ok(rows.into_iter().map(Project::from).collect())
    }

    pub async fn get_by_id(pool: &DbPool, user_id: i64, id: i64) -> ServiceResult<Project> {
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

    pub async fn create(
        pool: &DbPool,
        user_id: i64,
        input: &CreateProject,
    ) -> ServiceResult<Project> {
        let mut conn = pool.get().await?;

        let max_pos: Option<i32> = projects::table
            .filter(projects::user_id.eq(user_id))
            .select(max(projects::position))
            .first(&mut conn)
            .await?;
        let position = max_pos.unwrap_or(-1) + 1;

        let vt = input.view_type.as_ref().unwrap_or(&ProjectViewType::List);

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

        Ok(Project::from(proj_row))
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
            view_type: input
                .view_type
                .as_ref()
                .map(|vt| ProjectViewTypeMapping::from(vt.clone())),
            position: input.position,
            color: input.color.as_deref(),
            status: input
                .status
                .as_ref()
                .map(|s| ProjectStatusMapping::from(s.clone())),
        };

        // Only update if there's something to change
        let has_changes = changeset.title.is_some()
            || changeset.description.is_some()
            || changeset.view_type.is_some()
            || changeset.position.is_some()
            || changeset.color.is_some()
            || changeset.status.is_some();

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
            .filter(projects::status.eq(ProjectStatusMapping::Active))
            .filter(
                sql::<diesel::sql_types::Bool>("lower(title) = lower(")
                    .bind::<Text, _>(title)
                    .sql(")"),
            )
            .select(projects::id)
            .first(&mut conn)
            .await
            .optional()?;
        Ok(id)
    }
}
