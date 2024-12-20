use core_error::core_errors::CoreErrors;
use sea_orm::{
    sqlx::types::chrono::{DateTime, Utc},
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DbBackend, DbConn, DbErr,
    DeleteResult, EntityTrait, FromQueryResult, IntoActiveModel, QueryFilter, RuntimeErr, Set,
    Statement,
};

use crate::entity::{
    sea_orm_active_enums::{AccessLevelType, TaskStatusType},
    tasks, user_access,
};

#[derive(Debug, FromQueryResult)]
pub struct TaskModelFromQueryResult {
    pub id: i32,
    pub project_id: i32,
    pub assigned_to_id: i32,
    pub assigned_to_name: String,
    pub status: TaskStatusType,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub created_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
}

pub struct TasksQuery;

impl TasksQuery {
    /// Creates a new task in the database and assigns it to a user with default access permissions.
    ///
    /// This function performs two primary actions:
    /// 1. Inserts a new task into the `tasks` table using a Common Table Expression (CTE).
    /// 2. Updates the `user_access` table to grant the assigned user full access to the newly created task.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `project_id` - The ID of the project to which the task belongs.
    /// * `title` - The title of the task.
    /// * `description` - A detailed description of the task.
    /// * `user_id` - The ID of the user to whom the task is assigned.
    ///
    /// # Returns
    ///
    /// * `Result<tasks::Model, CoreErrors>` - Returns the created task as a `tasks::Model` on success,
    ///   or a `CoreErrors` variant if an error occurs during task creation or data retrieval.
    pub async fn create_task(
        db: &DbConn,
        project_id: i32,
        title: String,
        description: String,
        user_id: i32,
    ) -> Result<tasks::Model, CoreErrors> {
        // Define the SQL query using Common Table Expressions (CTEs).
        let sql = r#"
                WITH new_task AS (
                    INSERT INTO tasks (project_id, assigned_to, status, title, description, created_at)
                    VALUES ($1, $2, 'pending', $3, $4, CURRENT_TIMESTAMP AT TIME ZONE 'UTC')
                    RETURNING id, project_id, assigned_to, status, title, description, created_at
                ),
                user_access AS (
                    INSERT INTO user_access (user_id, company_id, project_id, task_id, role_id, access_level, created_at)
                    SELECT $2, NULL, NULL, nt.id, 1, 'full', CURRENT_TIMESTAMP AT TIME ZONE 'UTC'
                    FROM new_task nt
                )
                SELECT id, project_id, assigned_to, status, title, description, created_at
                FROM new_task;
            "#;

        // Prepare the SQL statement with parameterized inputs to prevent SQL injection.
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            vec![
                project_id.into(),  // $1 - The ID of the project.
                user_id.into(),     // $2 - The ID of the assigned user.
                title.into(),       // $3 - The title of the task.
                description.into(), // $4 - The description of the task.
            ],
        );

        // Execute the SQL query and fetch the result.
        let task_result = db.query_one(stmt).await?;

        // Parse the query result into a `tasks::Model`.
        let task = if let Some(row) = task_result {
            tasks::Model {
                id: row.try_get("", "id")?, // The ID of the newly created task.
                project_id: row.try_get("", "project_id")?, // The project ID associated with the task.
                assigned_to: row.try_get("", "assigned_to")?, // The ID of the assigned user.
                status: TaskStatusType::Pending, // The initial status of the task ('pending').
                title: row.try_get("", "title")?, // The task title.
                description: row.try_get("", "description").ok(), // The task description (nullable).
                priority: None, // Task priority is not part of the query.
                created_at: row.try_get("", "created_at")?, // Timestamp of task creation.
                due_date: None, // Task due date is not part of the query.
            }
        } else {
            // Handle the case where no task was created or returned from the query.
            return Err(CoreErrors::DatabaseServiceError(
                "Failed to create task".to_string(),
            ));
        };

        // Return the created task as a `tasks::Model`.
        Ok(task)
    }

    /// Retrieves a user's task and their access level for a specific task.
    ///
    /// This function checks the user's role and permissions from the `user_company` and `user_access` tables
    /// using a composite query with Common Table Expressions (CTEs). It determines the user's access level
    /// to the specified task and retrieves the task details.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user.
    /// * `task_id` - The ID of the task.
    ///
    /// # Returns
    /// * `Result<Option<(tasks::Model, i32)>, CoreErrors>` - Returns the task details as a `tasks::Model`
    ///   along with the user's role ID if found. Returns `None` if no matching task or access level is found.
    ///
    /// # Errors
    /// * Returns `CoreErrors` if the database query fails or if data conversion errors occur.
    pub async fn get_user_task_with_access_lvl(
        db: &DbConn,
        user_id: i32,
        task_id: i32,
    ) -> Result<Option<(tasks::Model, i32)>, CoreErrors> {
        // SQL query with multiple CTEs to handle different levels of permissions
        let sql = r#"
        WITH user_role AS (
            -- Check user's role and project details from `user_company`
            SELECT 
                uc.role_id AS role_id,
                r.name AS role_name,
                t.id AS task_id,
                t.project_id AS project_id,
                t.title AS task_title,
                t.description AS task_description,
                t.status::TEXT AS task_status, -- Cast to TEXT
                t.priority AS task_priority,
                t.due_date AS task_due_date,
                t.created_at AS task_created_at
            FROM user_company uc
            JOIN roles r ON uc.role_id = r.id
            JOIN tasks t ON uc.company_id = (SELECT company_id FROM projects WHERE id = t.project_id)
            WHERE uc.user_id = $1 AND t.id = $2 AND uc.role_id <= 2

            UNION ALL

            -- Check user's role and permissions from `user_access` at the project level
            SELECT 
                ua.role_id AS role_id,
                r.name AS role_name,
                t.id AS task_id,
                t.project_id AS project_id,
                t.title AS task_title,
                t.description AS task_description,
                t.status::TEXT AS task_status,
                t.priority AS task_priority,
                t.due_date AS task_due_date,
                t.created_at AS task_created_at
            FROM user_access ua
            JOIN roles r ON ua.role_id = r.id
            JOIN tasks t ON ua.project_id = t.project_id
            WHERE ua.user_id = $1 AND t.id = $2 AND ua.role_id <= 5

            UNION ALL

            -- Check user's role and permissions from `user_access` at the task level
            SELECT 
                ua.role_id AS role_id,
                r.name AS role_name,
                t.id AS task_id,
                t.project_id AS project_id,
                t.title AS task_title,
                t.description AS task_description,
                t.status::TEXT AS task_status,
                t.priority AS task_priority,
                t.due_date AS task_due_date,
                t.created_at AS task_created_at
            FROM user_access ua
            JOIN roles r ON ua.role_id = r.id
            JOIN tasks t ON ua.task_id = t.id
            WHERE ua.user_id = $1 AND ua.task_id = $2
        )
        SELECT 
            ur.task_id AS task_id,
            ur.project_id AS project_id,
            ur.task_title AS task_title,
            ur.task_description AS task_description,
            ur.task_status AS task_status,
            ur.task_priority AS task_priority,
            ur.task_due_date AS task_due_date,
            ur.task_created_at AS task_created_at,
            ur.role_id AS role_id,
            ur.role_name AS role_name
        FROM user_role ur
        LIMIT 1;
    "#;

        // Prepare the SQL statement with parameters
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                user_id.into(), // $1 - User ID
                task_id.into(), // $2 - Task ID
            ],
        );

        // Execute the query and fetch the result
        let query_result = db.query_one(stmt).await?;

        // Parse the query result into a `tasks::Model` and user's role ID
        if let Some(row) = query_result {
            // Extract and convert the task status
            let task_status: Option<String> = row.try_get("", "task_status")?;
            let status = task_status
                .map(|s| TaskStatusType::try_from(Some(s)))
                .transpose()?
                .unwrap();

            // Construct the `tasks::Model` from the query result
            let task = tasks::Model {
                id: row.try_get("", "task_id")?,                       // Task ID
                project_id: row.try_get("", "project_id")?,            // Project ID
                assigned_to: Some(user_id),                            // Assigned user ID
                status,                                                // Task status
                title: row.try_get("", "task_title")?,                 // Task title
                description: row.try_get("", "task_description").ok(), // Nullable description
                priority: row.try_get("", "task_priority").ok(),       // Nullable priority
                created_at: row.try_get("", "task_created_at")?,       // Creation timestamp
                due_date: row.try_get("", "task_due_date").ok(),       // Nullable due date
            };

            // Extract the user's role ID
            let role_id = row.try_get("", "role_id")?;

            // Return the task and role ID
            Ok(Some((task, role_id)))
        } else {
            // No matching task found, return `None`
            Ok(None)
        }
    }

    /// Adds a user to a task by creating or retrieving a `user_access` record.
    ///
    /// This function first checks if the user already has access to the specified task.
    /// If access exists, it returns the existing record. Otherwise, it creates a new `user_access`
    /// record with a default role and access level.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user to be added to the task.
    /// * `task_id` - The ID of the task to which the user should be added.
    ///
    /// # Returns
    /// * `Result<user_access::Model, CoreErrors>` - Returns the existing or newly created `user_access` record.
    ///
    /// # Errors
    /// * Returns `CoreErrors` if the database query or insert operation fails.
    pub async fn add_user_to_task(
        db: &DbConn,
        user_id: i32,
        task_id: i32,
    ) -> Result<user_access::Model, CoreErrors> {
        // Step 1: Check if the user already has access to the task
        if let Some(existing_access) = user_access::Entity::find()
            .filter(user_access::Column::UserId.eq(user_id)) // Filter by user ID
            .filter(user_access::Column::TaskId.eq(task_id)) // Filter by task ID
            .one(db)
            .await?
        {
            // If access exists, return the existing record
            return Ok(existing_access);
        }

        // Step 2: Define a new `user_access` record with default role and access level
        let new_access = user_access::ActiveModel {
            user_id: Set(user_id),                       // Assign the user ID
            task_id: Set(Some(task_id)),                 // Assign the task ID
            role_id: Set(Some(3)),                       // Assign role ID 3 (e.g., Manager)
            access_level: Set(AccessLevelType::Limited), // Set access level to 'Limited'
            ..Default::default()                         // Use default values for other fields
        };

        // Step 3: Insert the new `user_access` record into the database
        let inserted_access = new_access.insert(db).await?;

        // Step 4: Return the newly created `user_access` record
        Ok(inserted_access)
    }

    /// Removes a user's access to a task from the `user_access` table.
    ///
    /// Depending on the `request_user_id_lvl`, this function checks for a matching user-task association
    /// with or without a role level constraint. If an association exists, it deletes the record; otherwise,
    /// it returns an error.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user whose access to the task is being removed.
    /// * `task_id` - The ID of the task from which the user's access should be removed.
    /// * `request_user_id_lvl` - An optional role level constraint. If provided, only associations with a role
    ///   level greater than or equal to this value will be considered.
    ///
    /// # Returns
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` on successful removal or an error if no matching association is found.
    ///
    /// # Errors
    /// * Returns `CoreErrors::DatabaseServiceError` if the user-task association does not exist.
    /// * Returns `CoreErrors` for any database operation failures.
    pub async fn remove_user_from_task(
        db: &DbConn,
        user_id: i32,
        task_id: i32,
        request_user_id_lvl: Option<i32>,
    ) -> Result<(), CoreErrors> {
        // Step 1: Query the `user_access` table for a matching record
        let existing_access = match request_user_id_lvl {
            // Case 1: Query with role level constraint
            Some(level) => {
                user_access::Entity::find()
                    .filter(user_access::Column::UserId.eq(user_id)) // Match user ID
                    .filter(user_access::Column::TaskId.eq(task_id)) // Match task ID
                    .filter(user_access::Column::RoleId.gte(level)) // Role level constraint
                    .one(db)
                    .await?
            }
            // Case 2: Query without role level constraint
            None => {
                user_access::Entity::find()
                    .filter(user_access::Column::UserId.eq(user_id)) // Match user ID
                    .filter(user_access::Column::TaskId.eq(task_id)) // Match task ID
                    .one(db)
                    .await?
            }
        };

        // Step 2: Handle the query result
        match existing_access {
            Some(access) => {
                // Record exists, proceed to deletion
                let active_role = access.into_active_model(); // Convert to active model
                active_role.delete(db).await?; // Delete the record
            }
            None => {
                // No matching association found, return a meaningful error
                return Err(CoreErrors::DatabaseServiceError(
                    "User not associated with the specified task".to_string(),
                ));
            }
        };

        // Step 3: Return success
        Ok(())
    }

    /// Deletes a task from the database by its ID.
    ///
    /// This function checks if the task exists before attempting to delete it. If the task is found,
    /// it is deleted. If the task does not exist, an error is returned.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `task_id` - The ID of the task to be deleted.
    ///
    /// # Returns
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` if the task is successfully deleted,
    /// or an appropriate error if the task does not exist or the deletion fails.
    ///
    /// # Errors
    /// * Returns `CoreErrors::DatabaseServiceError` if the task does not exist.
    /// * Returns `CoreErrors` for any database operation failures.
    pub async fn delete_task(db: &DbConn, task_id: i32) -> Result<(), CoreErrors> {
        // Step 1: Attempt to find the task by its ID
        match tasks::Entity::find_by_id(task_id).one(db).await? {
            Some(task) => {
                // Step 2: Convert the found task entity into an active model for deletion
                let active_task = task.into_active_model();

                // Step 3: Delete the task record from the database
                active_task.delete(db).await?;
            }
            None => {
                // Step 4: Return an error if the task does not exist
                return Err(CoreErrors::DatabaseServiceError(format!(
                    "Task with ID {} does not exist",
                    task_id
                )));
            }
        }

        // Step 5: Return success if the task was deleted successfully
        Ok(())
    }

    /// Deletes all user associations for a specific task from the `user_access` table.
    ///
    /// This function removes all user-task associations linked to the specified task ID.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `task_id` - The ID of the task for which all user associations should be removed.
    ///
    /// # Returns
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` if the associations are successfully deleted,
    /// or an appropriate error if the deletion fails.
    ///
    /// # Errors
    /// * Returns `CoreErrors` for any database operation failures.
    pub async fn delete_all_users_from_task(db: &DbConn, task_id: i32) -> Result<(), CoreErrors> {
        let _: DeleteResult = user_access::Entity::delete_many()
            .filter(user_access::Column::TaskId.eq(task_id))
            .exec(db)
            .await?;

        Ok(())
    }

    /// Retrieves all tasks for a specific project that the user has access to.
    ///
    /// This function returns a list of tasks that a user can view or modify within a project.
    /// It checks for three types of access:
    /// 1. Company-wide access for roles with sufficient privileges (`role_id <= 3`).
    /// 2. Explicit project-level access for the user.
    /// 3. Explicit task-level access for the user.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection used for querying.
    /// * `project_id` - The ID of the project for which tasks are retrieved.
    /// * `user_id` - The ID of the user whose access is being verified.
    ///
    /// # Returns
    /// * `Result<Vec<TaskModelFromQueryResult>, CoreErrors>` -
    ///   * A vector of `TaskModelFromQueryResult` instances if successful.
    ///   * An error of type `CoreErrors` if the query execution fails.
    ///
    /// # Errors
    /// This function may return a `CoreErrors` variant in case of:
    /// * Database errors (e.g., connectivity issues, malformed SQL).
    /// * Invalid input causing query execution to fail.
    pub async fn get_all_project_tasks_by_access(
        db: &DbConn,
        project_id: i32,
        user_id: i32,
    ) -> Result<Vec<TaskModelFromQueryResult>, CoreErrors> {
        // SQL query to fetch tasks the user has access to based on:
        // 1. Company-wide access (`role_id <= 3` in `user_company`).
        // 2. Explicit project-level access (`user_access.project_id` matches).
        // 3. Explicit task-level access (`user_access.task_id` matches).
        let sql = r#"
        SELECT 
            t.id AS id,
            t.project_id AS project_id,
            u.id AS assigned_to_id,
            u.user_name AS assigned_to_name,
            t.status::TEXT AS status,
            t.title AS title,
            t.description AS description,
            t.priority AS priority,
            t.created_at AS created_at,
            t.due_date AS due_date
        FROM 
            tasks t
        LEFT JOIN 
            users u ON t.assigned_to = u.id
        WHERE 
            t.project_id = $1
            AND (
                -- Full access to the company (roles with `role_id <= 3`).
                EXISTS (
                    SELECT 1 
                    FROM user_company uc 
                    INNER JOIN projects p ON uc.company_id = p.company_id
                    WHERE uc.user_id = $2 
                      AND uc.role_id <= 3
                      AND p.id = $1
                )
                OR 
                -- Explicit project-level access granted to the user.
                EXISTS (
                    SELECT 1
                    FROM user_access ua
                    WHERE ua.user_id = $2
                      AND ua.project_id = $1
                )
                OR
                -- Explicit task-level access granted to the user.
                EXISTS (
                    SELECT 1
                    FROM user_access ua
                    WHERE ua.user_id = $2
                      AND ua.task_id = t.id
                )
            );
    "#;

        // Prepare the SQL statement with placeholders replaced by parameters.
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                project_id.into(), // Parameter $1: Project ID
                user_id.into(),    // Parameter $2: User ID
            ],
        );

        // Execute the query and map the result to a vector of `TaskModelFromQueryResult`.
        let tasks: Vec<TaskModelFromQueryResult> =
            TaskModelFromQueryResult::find_by_statement(stmt)
                .all(db)
                .await?;

        Ok(tasks)
    }
}

impl TryFrom<Option<String>> for TaskStatusType {
    type Error = DbErr;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        match value.as_deref() {
            Some("completed") => Ok(TaskStatusType::Completed),
            Some("in_progress") => Ok(TaskStatusType::InProgress),
            Some("pending") => Ok(TaskStatusType::Pending),
            _ => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid task_status value".into(),
            ))),
        }
    }
}
