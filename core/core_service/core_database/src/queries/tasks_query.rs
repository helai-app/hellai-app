use core_error::core_errors::CoreErrors;
use sea_orm::{ConnectionTrait, DatabaseBackend, DbConn, Statement};

use crate::entity::{sea_orm_active_enums::TaskStatusType, tasks};

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
}
