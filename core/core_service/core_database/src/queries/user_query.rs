use core_error::core_errors::CoreErrors;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbBackend, DbConn, EntityTrait, QueryFilter, QueryResult,
    Statement,
};

use crate::entity::{passwords, prelude, users};

pub struct UserQuery {}

impl UserQuery {
    pub async fn get_user_by_id(
        db: &DbConn,
        user_id: i32,
    ) -> Result<Option<users::Model>, CoreErrors> {
        let user = prelude::Users::find_by_id(user_id).one(db).await?;

        Ok(user)
    }

    pub async fn get_user_by_login(
        db: &DbConn,
        user_login: String,
    ) -> Result<Option<(users::Model, passwords::Model)>, CoreErrors> {
        let user_data = prelude::Users::find()
            .filter(users::Column::UserName.eq(user_login))
            .find_also_related(prelude::Passwords)
            .one(db)
            .await?;

        let user_with_password = match user_data {
            Some(user_data) => match user_data.1 {
                Some(password) => Some((user_data.0, password)),
                None => {
                    return Err(CoreErrors::DatabaseServiceError(
                        "user_data_broken".to_string(),
                    ))
                }
            },
            None => None,
        };

        Ok(user_with_password)
    }

    /// Creates a new user with password and assigns them a global role (id = 2).
    ///
    /// # Arguments
    ///
    /// * `db` - Reference to the database connection.
    /// * `user_name` - The username of the new user.
    /// * `user_password` - The password of the new user (stored as password_hash directly).
    /// * `user_email` - The email of the new user.
    ///
    /// # Returns
    ///
    /// A `Result<users::Model, CoreErrors>` containing the newly created user or an error.
    pub async fn create_new_user(
        db: &DbConn,
        user_name: String,
        user_password: String,
        user_email: Option<String>,
    ) -> Result<users::Model, CoreErrors> {
        // Construct the SQL statement with CTEs
        let sql = r#"
    WITH inserted_user AS (
        INSERT INTO users (username, email, is_active)
        VALUES ($1, $2, $3)
        RETURNING id, username, email, is_active, created_at, updated_at
    ),
    inserted_password AS (
        INSERT INTO passwords (user_id, password_hash)
        VALUES (
            (SELECT id FROM inserted_user),
            $4
        )
        RETURNING id
    ),
    inserted_user_role AS (
        INSERT INTO user_global_roles (user_id, global_role_id)
        VALUES (
            (SELECT id FROM inserted_user),
            $5
        )
        RETURNING id
    )
    SELECT 
        id,
        username,
        email,
        is_active,
        created_at,
        updated_at
    FROM inserted_user
    "#;

        // Define the global_role_id to assign
        let global_role_id = 2;

        // Create the statement with bound parameters
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [
                user_name.into(),
                user_email.unwrap_or_else(|| String::new()).into(),
                true.into(),
                user_password.into(),
                global_role_id.into(),
            ],
        );

        // Execute the query and get the result
        let new_user: Option<QueryResult> = db.query_one(stmt).await?;

        // Check that the result exists, otherwise return an error
        let new_user = new_user.ok_or(CoreErrors::DatabaseServiceError(
            "failed_create_user".to_string(),
        ))?;

        // Convert the result to a user model `users::Model`
        let user_model = users::Model {
            id: new_user.try_get("", "id")?,
            login: new_user.try_get("", "login")?,
            user_name: new_user.try_get("", "user_name")?,
            email: new_user.try_get("", "email")?,
            is_active: new_user.try_get("", "is_active")?,
            created_at: new_user.try_get("", "created_at")?,
            updated_at: new_user.try_get("", "updated_at")?,
        };

        Ok(user_model)
    }
}
