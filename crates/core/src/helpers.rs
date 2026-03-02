/// Fetch a single owned row by `id` + `user_id`, returning `ServiceError::NotFound`
/// when the row doesn't exist.
///
/// Usage:
/// ```ignore
/// get_owned!(pool, projects::table, projects::id, projects::user_id,
///            id, user_id, ProjectRow, "Project")
/// ```
macro_rules! get_owned {
    ($pool:expr, $table:expr, $id_col:expr, $uid_col:expr,
     $id:expr, $uid:expr, $row_type:ty, $name:literal) => {{
        let mut conn = $pool.get().await?;
        $table
            .filter($id_col.eq($id))
            .filter($uid_col.eq($uid))
            .select(<$row_type>::as_select())
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(|| $crate::ServiceError::NotFound(concat!($name, " not found").into()))
    }};
}

/// Delete a single owned row by `id` + `user_id`, returning `ServiceError::NotFound`
/// when no row was affected.
///
/// Usage:
/// ```ignore
/// delete_owned!(pool, projects::table, projects::id, projects::user_id,
///               id, user_id, "Project")
/// ```
macro_rules! delete_owned {
    ($pool:expr, $table:expr, $id_col:expr, $uid_col:expr,
     $id:expr, $uid:expr, $name:literal) => {{
        let mut conn = $pool.get().await?;
        let affected = diesel::delete($table.filter($id_col.eq($id)).filter($uid_col.eq($uid)))
            .execute(&mut conn)
            .await?;
        if affected == 0 {
            return Err($crate::ServiceError::NotFound(
                concat!($name, " not found").into(),
            ));
        }
        Ok(())
    }};
}

pub(crate) use delete_owned;
pub(crate) use get_owned;
