use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use north_db::models::{NewTag, NewTaskTag, TagRow};
use north_db::schema::{tags, task_tags};
use north_db::DbPool;
use north_dto::Tag;

use crate::ServiceResult;

pub struct TagService;

impl TagService {
    pub async fn list(pool: &DbPool, user_id: i64) -> ServiceResult<Vec<Tag>> {
        let mut conn = pool.get().await?;
        let rows = tags::table
            .inner_join(task_tags::table.on(task_tags::tag_id.eq(tags::id)))
            .filter(tags::user_id.eq(user_id))
            .select(TagRow::as_select())
            .distinct()
            .order(tags::name.asc())
            .load(&mut conn)
            .await?;
        Ok(rows.into_iter().map(Tag::from).collect())
    }

    /// Sync task tags: upsert tags by name, delete old task_tags, insert new ones.
    /// Caller must provide an active connection (for transaction support).
    pub async fn sync_task_tags(
        conn: &mut AsyncPgConnection,
        user_id: i64,
        task_id: i64,
        names: &[String],
    ) -> ServiceResult<()> {
        for name in names {
            diesel::insert_into(tags::table)
                .values(&NewTag {
                    user_id,
                    name,
                    color: "#6b7280",
                })
                .on_conflict((tags::user_id, tags::name))
                .do_nothing()
                .execute(conn)
                .await?;
        }

        diesel::delete(task_tags::table.filter(task_tags::task_id.eq(task_id)))
            .execute(conn)
            .await?;

        if !names.is_empty() {
            let tag_ids: Vec<i64> = tags::table
                .filter(tags::user_id.eq(user_id))
                .filter(tags::name.eq_any(names))
                .select(tags::id)
                .load(conn)
                .await?;

            let new_links: Vec<NewTaskTag> = tag_ids
                .into_iter()
                .map(|tag_id| NewTaskTag { task_id, tag_id })
                .collect();

            diesel::insert_into(task_tags::table)
                .values(&new_links)
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    /// Pool-level wrapper: gets a connection and calls sync_task_tags.
    pub async fn sync_task_tags_pooled(
        pool: &DbPool,
        user_id: i64,
        task_id: i64,
        names: &[String],
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        Self::sync_task_tags(&mut conn, user_id, task_id, names).await
    }

    /// Pool-level wrapper: gets a connection and calls add_task_tags.
    pub async fn add_task_tags_pooled(
        pool: &DbPool,
        user_id: i64,
        task_id: i64,
        names: &[String],
    ) -> ServiceResult<()> {
        let mut conn = pool.get().await?;
        Self::add_task_tags(&mut conn, user_id, task_id, names).await
    }

    /// Additive tag sync: upsert tags and add links without removing existing ones.
    pub async fn add_task_tags(
        conn: &mut AsyncPgConnection,
        user_id: i64,
        task_id: i64,
        names: &[String],
    ) -> ServiceResult<()> {
        if names.is_empty() {
            return Ok(());
        }

        for name in names {
            diesel::insert_into(tags::table)
                .values(&NewTag {
                    user_id,
                    name,
                    color: "#6b7280",
                })
                .on_conflict((tags::user_id, tags::name))
                .do_nothing()
                .execute(conn)
                .await?;
        }

        let tag_ids: Vec<i64> = tags::table
            .filter(tags::user_id.eq(user_id))
            .filter(tags::name.eq_any(names))
            .select(tags::id)
            .load(conn)
            .await?;

        let new_links: Vec<NewTaskTag> = tag_ids
            .into_iter()
            .map(|tag_id| NewTaskTag { task_id, tag_id })
            .collect();

        diesel::insert_into(task_tags::table)
            .values(&new_links)
            .on_conflict((task_tags::task_id, task_tags::tag_id))
            .do_nothing()
            .execute(conn)
            .await?;

        Ok(())
    }
}
