use sea_orm::{DbBackend, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Existing table creation code remains the same...
        // [Tables creation code omitted for brevity]

        // After creating tables, insert initial data

        // Insert GlobalRoles: admin, user, tester
        manager.get_connection().execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO "global_roles" ("name", "description") VALUES ($1, $2), ($3, $4), ($5, $6)"#,
            vec![
                "admin".into(),
                "Administrator with full access".into(),
                "user".into(),
                "Regular user".into(),
                "tester".into(),
                "Tester with limited access".into(),
            ],
        )).await?;

        // Insert test user: admin
        // Note: For security, we should hash the password 'admin'. For this example, we'll use a placeholder hash.
        // In a real application, use a proper password hashing algorithm like bcrypt or argon2.
        let password_hash = "hashed_admin_password"; // Replace with actual hash
        let salt = "random_salt"; // Replace with actual salt

        // Insert user into Users table
        manager.get_connection().execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO "users" ("username", "email", "is_active") VALUES ($1, $2, $3) RETURNING "id""#,
            vec![
                "admin".into(),
                "admin@example.com".into(),
                true.into(),
            ],
        )).await?;

        // Retrieve the generated user ID
        let user_id: i32 = manager
            .get_connection()
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT "id" FROM "users" WHERE "username" = $1"#,
                vec!["admin".into()],
            ))
            .await?
            .ok_or(DbErr::Custom("Failed to retrieve user ID".to_owned()))?
            .try_get("", "id")?;

        // Insert password into Passwords table
        manager.get_connection().execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO "passwords" ("user_id", "password_hash", "salt") VALUES ($1, $2, $3)"#,
            vec![
                user_id.into(),
                password_hash.into(),
                salt.into(),
            ],
        )).await?;

        // Assign global role 'admin' to the user
        // Retrieve the 'admin' role ID from GlobalRoles
        let global_role_id: i32 = manager
            .get_connection()
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT "id" FROM "global_roles" WHERE "name" = $1"#,
                vec!["admin".into()],
            ))
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve global role ID".to_owned(),
            ))?
            .try_get("", "id")?;

        // Insert into UserGlobalRoles
        manager
            .get_connection()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"INSERT INTO "user_global_roles" ("user_id", "global_role_id") VALUES ($1, $2)"#,
                vec![user_id.into(), global_role_id.into()],
            ))
            .await?;

        // Insert company: my_company
        manager
            .get_connection()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"INSERT INTO "companies" ("name") VALUES ($1) RETURNING "id""#,
                vec!["my_company".into()],
            ))
            .await?;

        // Retrieve the company ID
        let company_id: i32 = manager
            .get_connection()
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT "id" FROM "companies" WHERE "name" = $1"#,
                vec!["my_company".into()],
            ))
            .await?
            .ok_or(DbErr::Custom("Failed to retrieve company ID".to_owned()))?
            .try_get("", "id")?;

        // Assign user to company in UserCompanies
        manager
            .get_connection()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"INSERT INTO "user_companies" ("user_id", "company_id") VALUES ($1, $2)"#,
                vec![user_id.into(), company_id.into()],
            ))
            .await?;

        // Insert CompanyRoles: admin, user
        manager.get_connection().execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO "company_roles" ("company_id", "name", "description") VALUES ($1, $2, $3), ($1, $4, $5)"#,
            vec![
                company_id.into(),
                "admin".into(),
                "Company administrator".into(),
                "user".into(),
                "Company regular user".into(),
            ],
        )).await?;

        // Retrieve the company role ID for 'admin'
        let company_role_id: i32 = manager
            .get_connection()
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT "id" FROM "company_roles" WHERE "company_id" = $1 AND "name" = $2"#,
                vec![company_id.into(), "admin".into()],
            ))
            .await?
            .ok_or(DbErr::Custom(
                "Failed to retrieve company role ID".to_owned(),
            ))?
            .try_get("", "id")?;

        // Assign company role 'admin' to the user
        manager.get_connection().execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO "user_company_roles" ("user_id", "company_role_id") VALUES ($1, $2)"#,
            vec![
                user_id.into(),
                company_role_id.into(),
            ],
        )).await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation
        // [Tables drop code remains the same]
        Ok(())
    }
}
