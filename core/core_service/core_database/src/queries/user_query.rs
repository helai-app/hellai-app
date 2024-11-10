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

    /// Creates a new user in the database with associated password information.
    ///
    /// This function inserts a new user record into the `users` table along with
    /// their hashed password in the `passwords` table. The `is_active` status is set
    /// to `true` by default for all new users. It returns a user model containing
    /// essential user information such as ID, login, name, email, and timestamps.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection, used to execute the query.
    /// * `user_login` - The unique login identifier for the new user.
    /// * `user_name` - The full name of the new user.
    /// * `user_password` - The hashed password of the new user.
    /// * `user_email` - The email address associated with the new user.
    ///
    /// # Returns
    ///
    /// Returns a `Result<users::Model, CoreErrors>`, where:
    /// - On success, it contains the newly created `users::Model` with user details.
    /// - On failure, it returns a `CoreErrors::DatabaseServiceError` with a relevant error message.

    pub async fn create_new_user(
        db: &DbConn,
        user_login: String,
        user_name: String,
        user_password: String,
        user_email: String,
    ) -> Result<users::Model, CoreErrors> {
        // Define the SQL statement with CTEs for inserting the user and password
        let sql = r#"
        WITH inserted_user AS (
            INSERT INTO users (login, user_name, email, is_active)
            VALUES ($1, $2, $3, $4)
            RETURNING id, login, user_name, email, is_active, created_at, updated_at
        ),
        inserted_password AS (
            INSERT INTO passwords (user_id, password_hash)
            VALUES (
                (SELECT id FROM inserted_user),
                $5
            )
            RETURNING id
        )
        SELECT 
            id,
            login,
            user_name,
            email,
            is_active,
            created_at,
            updated_at
        FROM inserted_user
        "#;

        // Prepare the SQL statement with bound parameters
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [
                user_login.into(),
                user_name.into(),
                user_email.into(),
                true.into(),          // is_active is set to true for new users by default
                user_password.into(), // password hash should be passed instead of plain password
            ],
        );

        // Execute the query and get the result
        let new_user: Option<QueryResult> = db.query_one(stmt).await?;

        // Check if the result exists, otherwise return a custom error
        let new_user = new_user.ok_or(CoreErrors::DatabaseServiceError(
            "User creation failed: No user record returned from database.".to_string(),
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
