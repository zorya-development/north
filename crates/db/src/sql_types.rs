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

impl From<UserRoleMapping> for north_domain::UserRole {
    fn from(val: UserRoleMapping) -> Self {
        match val {
            UserRoleMapping::Admin => north_domain::UserRole::Admin,
            UserRoleMapping::User => north_domain::UserRole::User,
        }
    }
}

impl From<north_domain::UserRole> for UserRoleMapping {
    fn from(val: north_domain::UserRole) -> Self {
        match val {
            north_domain::UserRole::Admin => UserRoleMapping::Admin,
            north_domain::UserRole::User => UserRoleMapping::User,
        }
    }
}

impl From<ProjectViewTypeMapping> for north_domain::ProjectViewType {
    fn from(val: ProjectViewTypeMapping) -> Self {
        match val {
            ProjectViewTypeMapping::List => north_domain::ProjectViewType::List,
            ProjectViewTypeMapping::Kanban => north_domain::ProjectViewType::Kanban,
        }
    }
}

impl From<north_domain::ProjectViewType> for ProjectViewTypeMapping {
    fn from(val: north_domain::ProjectViewType) -> Self {
        match val {
            north_domain::ProjectViewType::List => ProjectViewTypeMapping::List,
            north_domain::ProjectViewType::Kanban => ProjectViewTypeMapping::Kanban,
        }
    }
}

impl From<ProjectStatusMapping> for north_domain::ProjectStatus {
    fn from(val: ProjectStatusMapping) -> Self {
        match val {
            ProjectStatusMapping::Active => north_domain::ProjectStatus::Active,
            ProjectStatusMapping::Archived => north_domain::ProjectStatus::Archived,
        }
    }
}

impl From<north_domain::ProjectStatus> for ProjectStatusMapping {
    fn from(val: north_domain::ProjectStatus) -> Self {
        match val {
            north_domain::ProjectStatus::Active => ProjectStatusMapping::Active,
            north_domain::ProjectStatus::Archived => ProjectStatusMapping::Archived,
        }
    }
}
