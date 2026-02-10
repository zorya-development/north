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
