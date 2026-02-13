// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "project_view_type"))]
    pub struct ProjectViewType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;
}

diesel::table! {
    images (id) {
        id -> Int8,
        user_id -> Int8,
        task_id -> Nullable<Int8>,
        path -> Text,
        filename -> Text,
        content_type -> Text,
        size_bytes -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    project_columns (id) {
        id -> Int8,
        project_id -> Int8,
        name -> Text,
        color -> Text,
        position -> Int4,
        is_done -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ProjectViewType;

    projects (id) {
        id -> Int8,
        user_id -> Int8,
        title -> Text,
        description -> Nullable<Text>,
        color -> Text,
        view_type -> ProjectViewType,
        position -> Int4,
        archived -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    saved_filters (id) {
        id -> Int8,
        user_id -> Int8,
        title -> Text,
        query -> Text,
        position -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    tags (id) {
        id -> Int8,
        user_id -> Int8,
        name -> Text,
        color -> Text,
    }
}

diesel::table! {
    task_tags (task_id, tag_id) {
        task_id -> Int8,
        tag_id -> Int8,
    }
}

diesel::table! {
    tasks (id) {
        id -> Int8,
        project_id -> Nullable<Int8>,
        parent_id -> Nullable<Int8>,
        column_id -> Nullable<Int8>,
        user_id -> Int8,
        title -> Text,
        body -> Nullable<Text>,
        sequential_limit -> Int2,
        start_at -> Nullable<Timestamptz>,
        due_date -> Nullable<Date>,
        completed_at -> Nullable<Timestamptz>,
        reviewed_at -> Nullable<Date>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        sort_key -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserRole;

    users (id) {
        id -> Int8,
        email -> Text,
        password_hash -> Text,
        name -> Text,
        role -> UserRole,
        settings -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(images -> tasks (task_id));
diesel::joinable!(images -> users (user_id));
diesel::joinable!(project_columns -> projects (project_id));
diesel::joinable!(projects -> users (user_id));
diesel::joinable!(saved_filters -> users (user_id));
diesel::joinable!(tags -> users (user_id));
diesel::joinable!(task_tags -> tags (tag_id));
diesel::joinable!(task_tags -> tasks (task_id));
diesel::joinable!(tasks -> project_columns (column_id));
diesel::joinable!(tasks -> projects (project_id));
diesel::joinable!(tasks -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    images,project_columns,projects,saved_filters,tags,task_tags,tasks,users,);
