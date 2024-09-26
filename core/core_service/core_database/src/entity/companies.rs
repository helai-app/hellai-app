//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "companies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::company_roles::Entity")]
    CompanyRoles,
    #[sea_orm(has_many = "super::user_companies::Entity")]
    UserCompanies,
}

impl Related<super::company_roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompanyRoles.def()
    }
}

impl Related<super::user_companies::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserCompanies.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}