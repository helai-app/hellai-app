use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
// use sea_orm_migration::prelude::Value;
use sea_orm::sea_query::Value;

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Begin a transaction to ensure atomicity
        let txn = manager.get_connection().begin().await?;

        // 1. Insert a test user
        let user_id: i32 = {
            // Hash the password (replace with your hashing function)
            let password_data = service::password_validation::hash_password("Admin123")
                .expect("Can't get hash password");
            let password_hash = password_data.0;

            // Insert user into Users table and retrieve the generated user ID
            let user = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    INSERT INTO "users" ("login", "user_name", "email", "is_active")
                    VALUES ($1, $2, $3, $4) RETURNING "id"
                    "#,
                    vec![
                        "admin".into(),
                        "Admin User".into(),
                        "admin@example.com".into(),
                        true.into(),
                    ],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert admin user".to_owned()))?;
            let user_id: i32 = user.try_get("", "id")?;

            // Insert password into Passwords table
            txn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"INSERT INTO "passwords" ("user_id", "password_hash") VALUES ($1, $2)"#,
                vec![user_id.into(), password_hash.into()],
            ))
            .await?;

            user_id
        };

        // 2. Insert a company and assign the user to it
        let company_id: i32 = {
            let company = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    INSERT INTO "companies" ("name", "name_alias", "description")
                    VALUES ($1, $2, $3) RETURNING "id"
                    "#,
                    vec![
                        "Example Company".into(),
                        "examplecompany".into(),
                        "A test company".into(),
                    ],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert company".to_owned()))?;
            let company_id: i32 = company.try_get("", "id")?;

            // Assign user to the company with a role and access level
            let role_id =
                insert_role(&txn, "Administrator", "Company administrator", None, 1).await?;
            txn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                    INSERT INTO "user_company" ("user_id", "company_id", "role_id", "access_level")
                    VALUES ($1, $2, $3, $4::access_level_type)
                    "#,
                vec![
                    user_id.into(),
                    company_id.into(),
                    role_id.into(),
                    "full".into(),
                ],
            ))
            .await?;

            company_id
        };

        // 3. Insert a project and assign the user to it
        let project_id: i32 = {
            let project = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    INSERT INTO "projects" ("company_id", "title", "description")
                    VALUES ($1, $2, $3) RETURNING "id"
                    "#,
                    vec![
                        company_id.into(),
                        "My Project".into(),
                        "A test project".into(),
                    ],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert project".to_owned()))?;
            let project_id: i32 = project.try_get("", "id")?;

            // Assign user to the project
            txn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO "user_access" ("user_id", "project_id", "access_level")
                VALUES ($1, $2, $3::access_level_type)
                "#,
                vec![user_id.into(), project_id.into(), "full".into()],
            ))
            .await?;

            project_id
        };

        // 4. Insert two tasks under the project
        let (task1_id, _) = {
            // Task 1
            let task1 = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
            INSERT INTO "tasks" (
                "project_id", "assigned_to", "status", "title", "description"
            )
            VALUES ($1, $2, $3::task_status_type, $4, $5) RETURNING "id"
            "#,
                    vec![
                        project_id.into(),
                        user_id.into(),
                        "pending".into(),
                        "Task One".into(),
                        "First test task".into(),
                    ],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert task one".to_owned()))?;
            let task1_id: i32 = task1.try_get("", "id")?;

            // Task 2
            let task2 = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
            INSERT INTO "tasks" (
                "project_id", "assigned_to", "status", "title", "description"
            )
            VALUES ($1, $2, $3::task_status_type, $4, $5) RETURNING "id"
            "#,
                    vec![
                        project_id.into(),
                        user_id.into(),
                        "pending".into(),
                        "Task Two".into(),
                        "Second test task".into(),
                    ],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert task two".to_owned()))?;
            let task2_id: i32 = task2.try_get("", "id")?;

            (task1_id, task2_id)
        };

        // 5. Insert a subtask under task1
        {
            let subtask = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
            INSERT INTO "subtasks" (
                "task_id", "assigned_to", "status", "title", "description"
            )
            VALUES ($1, $2, $3::task_status_type, $4, $5) RETURNING "id"
            "#,
                    vec![
                        task1_id.into(),
                        user_id.into(),
                        "pending".into(),
                        "Subtask One".into(),
                        "First subtask under Task One".into(),
                    ],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert subtask".to_owned()))?;
            let subtask_id: i32 = subtask.try_get("", "id")?;
            subtask_id
        };

        // 6. Create two notes: one personal note and one linked to task1
        {
            // Personal note
            txn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO "notes" ("user_id", "content", "created_at")
                VALUES ($1, $2, NOW())
                "#,
                vec![user_id.into(), "This is a personal note.".into()],
            ))
            .await?;

            // Note linked to task1
            txn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO "notes" ("user_id", "task_id", "content", "created_at")
                VALUES ($1, $2, $3, NOW())
                "#,
                vec![
                    user_id.into(),
                    task1_id.into(),
                    "This is a note linked to Task One.".into(),
                ],
            ))
            .await?;
        }

        // Commit the transaction
        txn.commit().await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        // For simplicity, we won't implement the down migration
        Ok(())
    }
}

// Helper functions

// Function to insert a role if it doesn't exist and return its ID
async fn insert_role(
    txn: &impl ConnectionTrait,
    name: &str,
    description: &str,
    parent_role_id: Option<i32>,
    level: i32,
) -> Result<i32, DbErr> {
    // Check if the role already exists
    if let Some(role) = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT "id" FROM "roles" WHERE "name" = $1"#,
            vec![name.into()],
        ))
        .await?
    {
        let role_id: i32 = role.try_get("", "id")?;
        Ok(role_id)
    } else {
        // Insert the role
        let role = txn
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                INSERT INTO "roles" ("name", "description", "parent_role_id", "level")
                VALUES ($1, $2, $3, $4) RETURNING "id"
                "#,
                vec![
                    name.into(),
                    description.into(),
                    parent_role_id.map_or(Value::Int(None), |id| id.into()),
                    level.into(),
                ],
            ))
            .await?
            .ok_or(DbErr::Custom("Failed to insert role".to_owned()))?;
        let role_id: i32 = role.try_get("", "id")?;
        Ok(role_id)
    }
}
