use sea_orm::{DbBackend, Statement};
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, TransactionTrait};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Existing table creation code remains the same...
        // [Tables creation code omitted for brevity]

        // Begin a transaction to ensure atomicity
        let txn = manager.get_connection().begin().await?;

        // 1. Insert GlobalRoles: admin, user, tester
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO "global_roles" ("name", "description") VALUES
                ($1, $2), ($3, $4), ($5, $6)
            "#,
            vec![
                "admin".into(),
                "Administrator with full access".into(),
                "user".into(),
                "Regular user".into(),
                "tester".into(),
                "Tester with limited access".into(),
            ],
        ))
        .await?;

        // 2. Retrieve the 'admin' global role ID
        let global_role_id: i32 = {
            let result = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "id" FROM "global_roles" WHERE "name" = $1"#,
                    vec!["admin".into()],
                ))
                .await?
                .ok_or(DbErr::Custom(
                    "Failed to retrieve global role ID".to_owned(),
                ))?;
            result.try_get("", "id")?
        };

        // 3. Insert test user: admin
        let password_data = service::password_validation::hash_password("Admin123")
            .expect("Can't get hash password");
        let password_hash = password_data.0;

        // Insert user into Users table and retrieve the generated user ID
        let user_id: i32 = {
            let result = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    INSERT INTO "users" ("username", "email", "is_active") VALUES
                        ($1, $2, $3) RETURNING "id"
                    "#,
                    vec!["admin".into(), "admin@example.com".into(), true.into()],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert admin user".to_owned()))?;
            result.try_get("", "id")?
        };

        // 4. Insert password into Passwords table
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO "passwords" ("user_id", "password_hash") VALUES ($1, $2)"#,
            vec![user_id.into(), password_hash.into()],
        ))
        .await?;

        // 5. Assign global role 'admin' to the user
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO "user_global_roles" ("user_id", "global_role_id") VALUES ($1, $2)
            "#,
            vec![user_id.into(), global_role_id.into()],
        ))
        .await?;

        // 6. Insert project: my_project and retrieve the project ID
        let project_id: i32 = {
            let result = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    INSERT INTO "projects" ("name") VALUES ($1) RETURNING "id"
                    "#,
                    vec!["my_project".into()],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert project".to_owned()))?;
            result.try_get("", "id")?
        };

        // 7. Assign user to project in UserProjects
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO "user_projects" ("user_id", "project_id") VALUES ($1, $2)
            "#,
            vec![user_id.into(), project_id.into()],
        ))
        .await?;

        // 8. Insert ProjectRoles: owner, administrator, user, guest
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO "project_roles" ("name", "description") VALUES
                ($1, $2), ($3, $4), ($5, $6), ($7, $8)
            "#,
            vec![
                ProjectRole::Owner.to_string().into(),
                "Project owner with full access".into(),
                ProjectRole::Administrator.to_string().into(),
                "Project administrator".into(),
                ProjectRole::User.to_string().into(),
                "Project regular user".into(),
                ProjectRole::Guest.to_string().into(),
                "Guest user with limited access".into(),
            ],
        ))
        .await?;

        // 9. Retrieve the project role ID for 'owner'
        let project_role_id: i32 = {
            let result = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    SELECT "id" FROM "project_roles" WHERE "name" = $1
                    "#,
                    vec![ProjectRole::Owner.to_string().into()],
                ))
                .await?
                .ok_or(DbErr::Custom(
                    "Failed to retrieve project role ID".to_owned(),
                ))?;
            result.try_get("", "id")?
        };

        // 10. Assign project role 'owner' to the user
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO "user_project_roles" ("user_id", "project_id", "project_role_id") VALUES ($1, $2, $3)
            "#,
            vec![user_id.into(), project_id.into(), project_role_id.into()],
        ))
        .await?;

        // Commit the transaction
        txn.commit().await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(Debug)]
enum ProjectRole {
    Owner = 0,
    Administrator = 1,
    User = 2,
    Guest = 3,
}

impl ToString for ProjectRole {
    fn to_string(&self) -> String {
        match self {
            ProjectRole::Owner => "owner".to_string(),
            ProjectRole::Administrator => "administrator".to_string(),
            ProjectRole::User => "user".to_string(),
            ProjectRole::Guest => "guest".to_string(),
        }
    }
}
