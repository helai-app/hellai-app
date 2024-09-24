use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Users
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Email).string().unique_key())
                    .col(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Passwords
        manager
            .create_table(
                Table::create()
                    .table(Passwords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Passwords::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Passwords::UserId).integer().not_null())
                    .col(ColumnDef::new(Passwords::PasswordHash).string().not_null())
                    .col(
                        ColumnDef::new(Passwords::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Passwords::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_passwords_user")
                            .from(Passwords::Table, Passwords::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. GlobalRoles
        manager
            .create_table(
                Table::create()
                    .table(GlobalRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GlobalRoles::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(GlobalRoles::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(GlobalRoles::Description).string())
                    .to_owned(),
            )
            .await?;

        // 4. UserGlobalRoles
        manager
            .create_table(
                Table::create()
                    .table(UserGlobalRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserGlobalRoles::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(UserGlobalRoles::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(UserGlobalRoles::GlobalRoleId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userglobalroles_user")
                            .from(UserGlobalRoles::Table, UserGlobalRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userglobalroles_globalrole")
                            .from(UserGlobalRoles::Table, UserGlobalRoles::GlobalRoleId)
                            .to(GlobalRoles::Table, GlobalRoles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_userglobalroles_user_role")
                            .col(UserGlobalRoles::UserId)
                            .col(UserGlobalRoles::GlobalRoleId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // 5. Companies
        manager
            .create_table(
                Table::create()
                    .table(Companies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Companies::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(Companies::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        // 6. UserCompanies
        manager
            .create_table(
                Table::create()
                    .table(UserCompanies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserCompanies::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(UserCompanies::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(UserCompanies::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserCompanies::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserCompanies::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_usercompanies_user")
                            .from(UserCompanies::Table, UserCompanies::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_usercompanies_company")
                            .from(UserCompanies::Table, UserCompanies::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_usercompanies_user_company")
                            .col(UserCompanies::UserId)
                            .col(UserCompanies::CompanyId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // 7. CompanyRoles (Define roles within a company)
        manager
            .create_table(
                Table::create()
                    .table(CompanyRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CompanyRoles::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(CompanyRoles::CompanyId).integer().not_null())
                    .col(ColumnDef::new(CompanyRoles::Name).string().not_null())
                    .col(ColumnDef::new(CompanyRoles::Description).string())
                    .col(
                        ColumnDef::new(CompanyRoles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(CompanyRoles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_companyroles_company")
                            .from(CompanyRoles::Table, CompanyRoles::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_companyroles_company_name")
                            .col(CompanyRoles::CompanyId)
                            .col(CompanyRoles::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // 8. UserCompanyRoles (Assign users to roles within a company)
        manager
            .create_table(
                Table::create()
                    .table(UserCompanyRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserCompanyRoles::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(UserCompanyRoles::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserCompanyRoles::CompanyRoleId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_usercompanyroles_user")
                            .from(UserCompanyRoles::Table, UserCompanyRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_usercompanyroles_companyrole")
                            .from(UserCompanyRoles::Table, UserCompanyRoles::CompanyRoleId)
                            .to(CompanyRoles::Table, CompanyRoles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_usercompanyroles_user_role")
                            .col(UserCompanyRoles::UserId)
                            .col(UserCompanyRoles::CompanyRoleId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation
        manager
            .drop_table(Table::drop().table(UserCompanyRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CompanyRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserCompanies::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Companies::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserGlobalRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GlobalRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Passwords::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Definitions of table and column enums for code clarity
#[derive(Iden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Passwords {
    Table,
    Id,
    UserId,
    PasswordHash,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum GlobalRoles {
    Table,
    Id,
    Name,
    Description,
}

#[derive(Iden)]
enum UserGlobalRoles {
    Table,
    Id,
    UserId,
    GlobalRoleId,
}

#[derive(Iden)]
enum Companies {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum UserCompanies {
    Table,
    Id,
    UserId,
    CompanyId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum CompanyRoles {
    Table,
    Id,
    CompanyId,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum UserCompanyRoles {
    Table,
    Id,
    UserId,
    CompanyRoleId,
}
