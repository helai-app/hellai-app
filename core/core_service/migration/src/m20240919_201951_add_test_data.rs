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
        // Replace 'service::password_validation::hash_password' with your actual password hashing function
        let password_data = service::password_validation::hash_password("Admin123")
            .expect("Can't get hash password");
        let password_hash = password_data.0; // Actual hash

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

        // 6. Insert company: my_company and retrieve the company ID
        let company_id: i32 = {
            let result = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    INSERT INTO "companies" ("name") VALUES ($1) RETURNING "id"
                    "#,
                    vec!["my_company".into()],
                ))
                .await?
                .ok_or(DbErr::Custom("Failed to insert company".to_owned()))?;
            result.try_get("", "id")?
        };

        // 7. Assign user to company in UserCompanies
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO "user_companies" ("user_id", "company_id") VALUES ($1, $2)
            "#,
            vec![user_id.into(), company_id.into()],
        ))
        .await?;

        // 8. Insert CompanyRoles: admin, user
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO "company_roles" ("company_id", "name", "description") VALUES
                ($1, $2, $3), ($1, $4, $5)
            "#,
            vec![
                company_id.into(),
                "admin".into(),
                "Company administrator".into(),
                "user".into(),
                "Company regular user".into(),
            ],
        ))
        .await?;

        // 9. Retrieve the company role ID for 'admin'
        let company_role_id: i32 = {
            let result = txn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                    SELECT "id" FROM "company_roles" WHERE "company_id" = $1 AND "name" = $2
                    "#,
                    vec![company_id.into(), "admin".into()],
                ))
                .await?
                .ok_or(DbErr::Custom(
                    "Failed to retrieve company role ID".to_owned(),
                ))?;
            result.try_get("", "id")?
        };

        // 10. Assign company role 'admin' to the user
        // Updated to include "company_id" in the insert statement
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            INSERT INTO "user_company_roles" ("user_id", "company_id", "company_role_id") VALUES ($1, $2, $3)
            "#,
            vec![user_id.into(), company_id.into(), company_role_id.into()],
        ))
        .await?;

        // Commit the transaction
        txn.commit().await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        // Since we're focusing on the data insertion, and your table drop code remains the same,
        // we can leave this method unchanged.
        Ok(())
    }
}
