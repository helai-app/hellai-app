//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub company_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub decoration_color: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::companies::Entity",
        from = "Column::CompanyId",
        to = "super::companies::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Companies,
    #[sea_orm(has_many = "super::knowledge_base::Entity")]
    KnowledgeBase,
    #[sea_orm(has_many = "super::notes::Entity")]
    Notes,
    #[sea_orm(has_many = "super::tasks::Entity")]
    Tasks,
    #[sea_orm(has_many = "super::user_access::Entity")]
    UserAccess,
}

impl Related<super::companies::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Companies.def()
    }
}

impl Related<super::knowledge_base::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::KnowledgeBase.def()
    }
}

impl Related<super::notes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Notes.def()
    }
}

impl Related<super::tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tasks.def()
    }
}

impl Related<super::user_access::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAccess.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
