//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::project_roles::Entity")]
    ProjectRoles,
    #[sea_orm(has_many = "super::user_project_roles::Entity")]
    UserProjectRoles,
    #[sea_orm(has_many = "super::user_projects::Entity")]
    UserProjects,
}

impl Related<super::project_roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectRoles.def()
    }
}

impl Related<super::user_project_roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserProjectRoles.def()
    }
}

impl Related<super::user_projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserProjects.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
