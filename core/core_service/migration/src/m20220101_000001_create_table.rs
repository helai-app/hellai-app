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

        // 5. Projects
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(Projects::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        // 6. UserProjects
        manager
            .create_table(
                Table::create()
                    .table(UserProjects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserProjects::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(UserProjects::UserId).integer().not_null())
                    .col(ColumnDef::new(UserProjects::ProjectId).integer().not_null())
                    .col(
                        ColumnDef::new(UserProjects::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserProjects::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userprojects_user")
                            .from(UserProjects::Table, UserProjects::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userprojects_project")
                            .from(UserProjects::Table, UserProjects::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_userprojects_user_project")
                            .col(UserProjects::UserId)
                            .col(UserProjects::ProjectId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // 7. ProjectRoles (Define roles within a project without project_id)
        manager
            .create_table(
                Table::create()
                    .table(ProjectRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectRoles::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(ProjectRoles::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(ProjectRoles::Description).string())
                    .col(
                        ColumnDef::new(ProjectRoles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ProjectRoles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 8. UserProjectRoles (Assign users to roles within a project)
        manager
            .create_table(
                Table::create()
                    .table(UserProjectRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserProjectRoles::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(UserProjectRoles::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserProjectRoles::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserProjectRoles::ProjectRoleId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserProjectRoles::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserProjectRoles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userprojectroles_user")
                            .from(UserProjectRoles::Table, UserProjectRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userprojectroles_project")
                            .from(UserProjectRoles::Table, UserProjectRoles::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userprojectroles_projectrole")
                            .from(UserProjectRoles::Table, UserProjectRoles::ProjectRoleId)
                            .to(ProjectRoles::Table, ProjectRoles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_userprojectroles_user_project_role")
                            .col(UserProjectRoles::UserId)
                            .col(UserProjectRoles::ProjectId)
                            .col(UserProjectRoles::ProjectRoleId)
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
            .drop_table(Table::drop().table(UserProjectRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProjectRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserProjects::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
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
enum Projects {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum UserProjects {
    Table,
    Id,
    UserId,
    ProjectId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ProjectRoles {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum UserProjectRoles {
    Table,
    Id,
    UserId,
    ProjectId,
    ProjectRoleId,
    CreatedAt,
    UpdatedAt,
}
