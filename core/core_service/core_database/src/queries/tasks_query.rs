use core_error::core_errors::CoreErrors;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DbBackend, DbConn, DbErr,
    EntityTrait, QueryFilter, RuntimeErr, Set, Statement,
};

use crate::entity::{
    sea_orm_active_enums::{AccessLevelType, TaskStatusType},
    tasks, user_access,
};

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

    pub async fn get_user_task_with_access_lvl(
        db: &DbConn,
        user_id: i32,
        task_id: i32,
    ) -> Result<Option<(tasks::Model, i32)>, CoreErrors> {
        // SQL query to retrieve the task details and the user's role for a user
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
            WHERE uc.user_id = $1 
            AND t.id = $2
            AND uc.role_id <= 2

            UNION ALL

            -- Check user's role and project details from `user_access` for project-level permissions
            SELECT 
                ua.role_id AS role_id,
                r.name AS role_name,
                t.id AS task_id,
                t.project_id AS project_id,
                t.title AS task_title,
                t.description AS task_description,
                t.status::TEXT AS task_status, -- Cast to TEXT
                t.priority AS task_priority,
                t.due_date AS task_due_date,
                t.created_at AS task_created_at
            FROM user_access ua
            JOIN roles r ON ua.role_id = r.id
            JOIN tasks t ON ua.project_id = t.project_id
            WHERE ua.user_id = $1 
            AND t.id = $2
            AND ua.role_id <= 5

            UNION ALL

            -- Check user's role and project details from `user_access` for task-level permissions
            SELECT 
                ua.role_id AS role_id,
                r.name AS role_name,
                t.id AS task_id,
                t.project_id AS project_id,
                t.title AS task_title,
                t.description AS task_description,
                t.status::TEXT AS task_status, -- Cast to TEXT
                t.priority AS task_priority,
                t.due_date AS task_due_date,
                t.created_at AS task_created_at
            FROM user_access ua
            JOIN roles r ON ua.role_id = r.id
            JOIN tasks t ON ua.task_id = t.id
            WHERE ua.user_id = $1 
            AND ua.task_id = $2
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
                user_id.into(), // $1 - The user ID
                task_id.into(), // $2 - The task ID
            ],
        );

        // Execute the query and retrieve the result
        let query_result = db.query_one(stmt).await?;

        // Parse the query result into a `tasks::Model` and user role ID if found
        if let Some(row) = query_result {
            let task_status: Option<String> = row.try_get("", "task_status")?;
            let status = task_status
                .map(|s| TaskStatusType::try_from(Some(s)))
                .transpose()?
                .unwrap();

            let task = tasks::Model {
                id: row.try_get("", "task_id")?,                       // Task ID
                project_id: row.try_get("", "project_id")?, // Project ID associated with the task
                assigned_to: Some(user_id), // Assigned to user (as inferred from context)
                status: status,             // Task status
                title: row.try_get("", "task_title")?, // Task title
                description: row.try_get("", "task_description").ok(), // Nullable task description
                priority: row.try_get("", "task_priority").ok(), // Nullable task priority
                created_at: row.try_get("", "task_created_at")?, // Task creation timestamp
                due_date: row.try_get("", "task_due_date").ok(), // Nullable task due date
            };
            let role_id = row.try_get("", "role_id")?; // User's role ID
            Ok(Some((task, role_id)))
        } else {
            // Return `None` if no matching task is found
            Ok(None)
        }
    }

    pub async fn add_user_to_task(
        db: &DbConn,
        user_id: i32,
        task_id: i32,
    ) -> Result<user_access::Model, CoreErrors> {
        // Step 1: Check if the `user_access` entry already exists
        if let Some(existing_access) = user_access::Entity::find()
            .filter(user_access::Column::UserId.eq(user_id))
            .filter(user_access::Column::TaskId.eq(task_id))
            .one(db)
            .await?
        {
            // Return the existing record if found
            return Ok(existing_access);
        }

        // Step 2: Create a new `user_access` record with the specified details
        let new_access = user_access::ActiveModel {
            user_id: Set(user_id),                       // Set the user ID
            task_id: Set(Some(task_id)),                 // Associate with the specified project
            role_id: Set(Some(3)),                       // Assign role ID 3 (Manager)
            access_level: Set(AccessLevelType::Limited), // Set access level to limited
            ..Default::default()                         // Use default values for other fields
        };

        // Step 3: Insert the new role assignment into the database
        let result = new_access.insert(db).await?;

        // Step 4: Return the successfully created `user_access` record
        Ok(result)
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
