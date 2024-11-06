use extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create ENUM types for access_level_type and task_status_type
        manager
            .create_type(
                Type::create()
                    .as_enum(AccessLevelType::Table)
                    .values([
                        AccessLevelType::Full,
                        AccessLevelType::Limited,
                        AccessLevelType::Restricted,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(TaskStatusType::Table)
                    .values([
                        TaskStatusType::Pending,
                        TaskStatusType::InProgress,
                        TaskStatusType::Completed,
                    ])
                    .to_owned(),
            )
            .await?;

        // 1. Users table
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
                        ColumnDef::new(Users::Login)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::UserName).string().not_null())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
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

        // 2. Passwords table
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

        manager
            .create_index(
                Index::create()
                    .name("idx_passwords_user_id")
                    .table(Passwords::Table)
                    .col(Passwords::UserId)
                    .to_owned(),
            )
            .await?;

        // 3. Companies table
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
                    .col(ColumnDef::new(Companies::Name).string().not_null())
                    .col(
                        ColumnDef::new(Companies::NameAlias)
                            .string()
                            .not_null()
                            .unique_key()
                            .check(Expr::cust("name_alias ~ '^[a-z]+$'")),
                    )
                    .col(ColumnDef::new(Companies::Description).string().null())
                    .col(ColumnDef::new(Companies::ContactInfo).string().null())
                    .to_owned(),
            )
            .await?;

        // 4. Roles table
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Roles::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Roles::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Roles::Description).string().null())
                    .col(ColumnDef::new(Roles::ParentRoleId).integer().null())
                    .col(
                        ColumnDef::new(Roles::Level)
                            .integer()
                            .not_null()
                            .check(Expr::cust("level > 0")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_roles_parent_role")
                            .from(Roles::Table, Roles::ParentRoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // 5. UserCompany table
        manager
            .create_table(
                Table::create()
                    .table(UserCompany::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserCompany::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(UserCompany::UserId).integer().not_null())
                    .col(ColumnDef::new(UserCompany::CompanyId).integer().not_null())
                    .col(ColumnDef::new(UserCompany::RoleId).integer().not_null())
                    .col(
                        ColumnDef::new(UserCompany::AccessLevel)
                            .enumeration(
                                AccessLevelType::Table,
                                [
                                    AccessLevelType::Full,
                                    AccessLevelType::Limited,
                                    AccessLevelType::Restricted,
                                ],
                            )
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_usercompany_user")
                            .from(UserCompany::Table, UserCompany::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_usercompany_company")
                            .from(UserCompany::Table, UserCompany::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_usercompany_role")
                            .from(UserCompany::Table, UserCompany::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_usercompany_user_company")
                            .col(UserCompany::UserId)
                            .col(UserCompany::CompanyId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // 6. Projects table
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
                    .col(ColumnDef::new(Projects::CompanyId).integer().not_null())
                    .col(ColumnDef::new(Projects::Title).string().not_null())
                    .col(ColumnDef::new(Projects::Description).string().null())
                    .col(
                        ColumnDef::new(Projects::DecorationColor)
                            .string()
                            .null()
                            .check(Expr::cust("decoration_color ~ '^#[0-9A-Fa-f]{6}$'")),
                    )
                    .col(
                        ColumnDef::new(Projects::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Projects::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_projects_company")
                            .from(Projects::Table, Projects::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_company_id")
                    .table(Projects::Table)
                    .col(Projects::CompanyId)
                    .to_owned(),
            )
            .await?;

        // 7. Tasks table
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tasks::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Tasks::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Tasks::AssignedTo).integer().null())
                    .col(
                        ColumnDef::new(Tasks::Status)
                            .enumeration(
                                TaskStatusType::Table,
                                [
                                    TaskStatusType::Pending,
                                    TaskStatusType::InProgress,
                                    TaskStatusType::Completed,
                                ],
                            )
                            .not_null()
                            .default(Expr::cust("'pending'::task_status_type")),
                    )
                    .col(ColumnDef::new(Tasks::Title).string().not_null())
                    .col(ColumnDef::new(Tasks::Description).string().null())
                    .col(ColumnDef::new(Tasks::Priority).string().null())
                    .col(
                        ColumnDef::new(Tasks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Tasks::DueDate)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_project")
                            .from(Tasks::Table, Tasks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_assigned_to")
                            .from(Tasks::Table, Tasks::AssignedTo)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_project_id")
                    .table(Tasks::Table)
                    .col(Tasks::ProjectId)
                    .to_owned(),
            )
            .await?;

        // 8. Subtasks table
        manager
            .create_table(
                Table::create()
                    .table(Subtasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Subtasks::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Subtasks::TaskId).integer().not_null())
                    .col(ColumnDef::new(Subtasks::AssignedTo).integer().null())
                    .col(
                        ColumnDef::new(Subtasks::Status)
                            .enumeration(
                                TaskStatusType::Table,
                                [
                                    TaskStatusType::Pending,
                                    TaskStatusType::InProgress,
                                    TaskStatusType::Completed,
                                ],
                            )
                            .not_null()
                            .default(Expr::cust("'pending'::task_status_type")),
                    )
                    .col(ColumnDef::new(Subtasks::Title).string().not_null())
                    .col(ColumnDef::new(Subtasks::Description).string().null())
                    .col(
                        ColumnDef::new(Subtasks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Subtasks::DueDate)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subtasks_task")
                            .from(Subtasks::Table, Subtasks::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subtasks_assigned_to")
                            .from(Subtasks::Table, Subtasks::AssignedTo)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_subtasks_task_id")
                    .table(Subtasks::Table)
                    .col(Subtasks::TaskId)
                    .to_owned(),
            )
            .await?;

        // 9. Notes table
        manager
            .create_table(
                Table::create()
                    .table(Notes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Notes::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Notes::UserId).integer().not_null())
                    .col(ColumnDef::new(Notes::CompanyId).integer().null())
                    .col(ColumnDef::new(Notes::ProjectId).integer().null())
                    .col(ColumnDef::new(Notes::TaskId).integer().null())
                    .col(ColumnDef::new(Notes::SubtaskId).integer().null())
                    .col(ColumnDef::new(Notes::Content).string().not_null())
                    .col(ColumnDef::new(Notes::Tags).string().null())
                    .col(
                        ColumnDef::new(Notes::DecorationColor)
                            .string()
                            .null()
                            .check(Expr::cust("decoration_color ~ '^#[0-9A-Fa-f]{6}$'")),
                    )
                    .col(
                        ColumnDef::new(Notes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notes_user")
                            .from(Notes::Table, Notes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Add foreign keys for optional relationships
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notes_company")
                            .from(Notes::Table, Notes::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notes_project")
                            .from(Notes::Table, Notes::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notes_task")
                            .from(Notes::Table, Notes::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notes_subtask")
                            .from(Notes::Table, Notes::SubtaskId)
                            .to(Subtasks::Table, Subtasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_notes_user_id")
                    .table(Notes::Table)
                    .col(Notes::UserId)
                    .to_owned(),
            )
            .await?;

        // Since we cannot add a table-level check constraint directly in SeaORM Migrations,
        // we can execute raw SQL using the connection.

        let sql = "ALTER TABLE notes ADD CONSTRAINT chk_notes_entity CHECK (
            (company_id IS NOT NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NULL) OR
            (company_id IS NULL AND project_id IS NOT NULL AND task_id IS NULL AND subtask_id IS NULL) OR
            (company_id IS NULL AND project_id IS NULL AND task_id IS NOT NULL AND subtask_id IS NULL) OR
            (company_id IS NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NOT NULL) OR
            (company_id IS NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NULL)
        );";

        manager
            .get_connection()
            .execute_unprepared(sql)
            .await
            .map(|_| ())?;

        // 10. KnowledgeBase table
        manager
            .create_table(
                Table::create()
                    .table(KnowledgeBase::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(KnowledgeBase::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(KnowledgeBase::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(KnowledgeBase::ProjectId).integer().null())
                    .col(ColumnDef::new(KnowledgeBase::Title).string().not_null())
                    .col(ColumnDef::new(KnowledgeBase::Content).string().null())
                    .col(
                        ColumnDef::new(KnowledgeBase::AccessLevel) // Add AccessLevel column
                            .enumeration(
                                AccessLevelType::Table,
                                [
                                    AccessLevelType::Full,
                                    AccessLevelType::Limited,
                                    AccessLevelType::Restricted,
                                ],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(KnowledgeBase::RoleId).integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_knowledgebase_company")
                            .from(KnowledgeBase::Table, KnowledgeBase::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_knowledgebase_project")
                            .from(KnowledgeBase::Table, KnowledgeBase::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_knowledgebase_role")
                            .from(KnowledgeBase::Table, KnowledgeBase::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_knowledgebase_company_id")
                    .table(KnowledgeBase::Table)
                    .col(KnowledgeBase::CompanyId)
                    .to_owned(),
            )
            .await?;

        // 11. UserAccess table
        manager
            .create_table(
                Table::create()
                    .table(UserAccess::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserAccess::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(UserAccess::UserId).integer().not_null())
                    .col(ColumnDef::new(UserAccess::CompanyId).integer().null())
                    .col(ColumnDef::new(UserAccess::ProjectId).integer().null())
                    .col(ColumnDef::new(UserAccess::TaskId).integer().null())
                    .col(ColumnDef::new(UserAccess::SubtaskId).integer().null())
                    .col(ColumnDef::new(UserAccess::RoleId).integer().null()) // New RoleId column
                    .col(
                        ColumnDef::new(UserAccess::AccessLevel)
                            .enumeration(
                                AccessLevelType::Table,
                                [
                                    AccessLevelType::Full,
                                    AccessLevelType::Limited,
                                    AccessLevelType::Restricted,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserAccess::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_useraccess_user")
                            .from(UserAccess::Table, UserAccess::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Add foreign keys for optional relationships
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_useraccess_company")
                            .from(UserAccess::Table, UserAccess::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_useraccess_project")
                            .from(UserAccess::Table, UserAccess::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_useraccess_task")
                            .from(UserAccess::Table, UserAccess::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_useraccess_subtask")
                            .from(UserAccess::Table, UserAccess::SubtaskId)
                            .to(Subtasks::Table, Subtasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_useraccess_role") // Foreign key for RoleId
                            .from(UserAccess::Table, UserAccess::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_useraccess_user_id")
                    .table(UserAccess::Table)
                    .col(UserAccess::UserId)
                    .to_owned(),
            )
            .await?;

        // Add check constraint to UserAccess table
        let sql = "ALTER TABLE user_access ADD CONSTRAINT chk_useraccess_level CHECK (
            (company_id IS NOT NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NULL) OR
            (company_id IS NULL AND project_id IS NOT NULL AND task_id IS NULL AND subtask_id IS NULL) OR
            (company_id IS NULL AND project_id IS NULL AND task_id IS NOT NULL AND subtask_id IS NULL) OR
            (company_id IS NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NOT NULL)
        );";
        manager
            .get_connection()
            .execute_unprepared(sql)
            .await
            .map(|_| ())?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(UserAccess::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(KnowledgeBase::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Notes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Subtasks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserCompany::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Companies::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Passwords::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        // Drop ENUM types
        manager
            .drop_type(Type::drop().name(AccessLevelType::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(TaskStatusType::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Definitions of table and column enums for code clarity

#[derive(Iden)]
enum AccessLevelType {
    Table,
    Full,
    Limited,
    Restricted,
}

#[derive(Iden)]
enum TaskStatusType {
    Table,
    Pending,
    InProgress,
    Completed,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Login,
    UserName,
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
enum Companies {
    Table,
    Id,
    Name,
    NameAlias,
    Description,
    ContactInfo,
}

#[derive(Iden)]
enum Roles {
    Table,
    Id,
    Name,
    Description,
    ParentRoleId,
    Level,
}

#[derive(Iden)]
enum UserCompany {
    Table,
    Id,
    UserId,
    CompanyId,
    RoleId,
    AccessLevel,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
    CompanyId,
    Title,
    Description,
    DecorationColor,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Tasks {
    Table,
    Id,
    ProjectId,
    AssignedTo,
    Status,
    Title,
    Description,
    Priority,
    CreatedAt,
    DueDate,
}

#[derive(Iden)]
enum Subtasks {
    Table,
    Id,
    TaskId,
    AssignedTo,
    Status,
    Title,
    Description,
    CreatedAt,
    DueDate,
}

#[derive(Iden)]
enum Notes {
    Table,
    Id,
    UserId,
    CompanyId,
    ProjectId,
    TaskId,
    SubtaskId,
    Content,
    Tags,
    DecorationColor,
    CreatedAt,
}

#[derive(Iden)]
enum KnowledgeBase {
    Table,
    Id,
    CompanyId,
    ProjectId,
    Title,
    Content,
    AccessLevel,
    RoleId,
}

#[derive(Iden)]
enum UserAccess {
    Table,
    Id,
    UserId,
    CompanyId,
    RoleId,
    ProjectId,
    TaskId,
    SubtaskId,
    AccessLevel,
    CreatedAt,
}
