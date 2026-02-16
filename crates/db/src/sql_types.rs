use diesel_derive_enum::DbEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::UserRole"]
pub enum UserRoleMapping {
    #[db_rename = "admin"]
    Admin,
    #[db_rename = "user"]
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ProjectViewType"]
pub enum ProjectViewTypeMapping {
    #[db_rename = "list"]
    List,
    #[db_rename = "kanban"]
    Kanban,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ProjectStatus"]
pub enum ProjectStatusMapping {
    #[db_rename = "active"]
    Active,
    #[db_rename = "archived"]
    Archived,
}

impl From<UserRoleMapping> for north_dto::UserRole {
    fn from(val: UserRoleMapping) -> Self {
        match val {
            UserRoleMapping::Admin => north_dto::UserRole::Admin,
            UserRoleMapping::User => north_dto::UserRole::User,
        }
    }
}

impl From<north_dto::UserRole> for UserRoleMapping {
    fn from(val: north_dto::UserRole) -> Self {
        match val {
            north_dto::UserRole::Admin => UserRoleMapping::Admin,
            north_dto::UserRole::User => UserRoleMapping::User,
        }
    }
}

impl From<ProjectViewTypeMapping> for north_dto::ProjectViewType {
    fn from(val: ProjectViewTypeMapping) -> Self {
        match val {
            ProjectViewTypeMapping::List => north_dto::ProjectViewType::List,
            ProjectViewTypeMapping::Kanban => north_dto::ProjectViewType::Kanban,
        }
    }
}

impl From<north_dto::ProjectViewType> for ProjectViewTypeMapping {
    fn from(val: north_dto::ProjectViewType) -> Self {
        match val {
            north_dto::ProjectViewType::List => ProjectViewTypeMapping::List,
            north_dto::ProjectViewType::Kanban => ProjectViewTypeMapping::Kanban,
        }
    }
}

impl From<ProjectStatusMapping> for north_dto::ProjectStatus {
    fn from(val: ProjectStatusMapping) -> Self {
        match val {
            ProjectStatusMapping::Active => north_dto::ProjectStatus::Active,
            ProjectStatusMapping::Archived => north_dto::ProjectStatus::Archived,
        }
    }
}

impl From<north_dto::ProjectStatus> for ProjectStatusMapping {
    fn from(val: north_dto::ProjectStatus) -> Self {
        match val {
            north_dto::ProjectStatus::Active => ProjectStatusMapping::Active,
            north_dto::ProjectStatus::Archived => ProjectStatusMapping::Archived,
        }
    }
}
