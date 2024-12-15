//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub description: Option<String>,
    pub parent_role_id: Option<i32>,
    pub level: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::knowledge_base::Entity")]
    KnowledgeBase,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentRoleId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    SelfRef,
    #[sea_orm(has_many = "super::user_access::Entity")]
    UserAccess,
    #[sea_orm(has_many = "super::user_company::Entity")]
    UserCompany,
}

impl Related<super::knowledge_base::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::KnowledgeBase.def()
    }
}

impl Related<super::user_access::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAccess.def()
    }
}

impl Related<super::user_company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserCompany.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
