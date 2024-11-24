use core_error::core_errors::CoreErrors;
use sea_orm::{ActiveModelTrait, DbBackend, DbConn, EntityTrait, IntoActiveModel, Set, Statement};

use crate::entity::notes;

/// Provides methods for querying and manipulating notes.
pub struct NotesQuery;

impl NotesQuery {
    /// Creates a new note in the database with the specified details.
    ///
    /// This function inserts a new record into the `notes` table, associating it with optional entities
    /// such as a company, project, task, or subtask.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user creating the note.
    /// * `project_id` - Optional ID of the project associated with the note.
    /// * `company_id` - Optional ID of the company associated with the note.
    /// * `task_id` - Optional ID of the task associated with the note.
    /// * `subtask_id` - Optional ID of the subtask associated with the note.
    /// * `content` - The content of the note.
    /// * `tags` - Tags associated with the note.
    /// * `decoration_color` - Decoration color for the note.
    ///
    /// # Returns
    ///
    /// * `Result<notes::Model, CoreErrors>` - Returns the created note model on success,
    ///   or a `CoreErrors` error if the operation fails.
    pub async fn create_note(
        db: &DbConn,
        user_id: i32,
        project_id: Option<i32>,
        company_id: Option<i32>,
        task_id: Option<i32>,
        subtask_id: Option<i32>,
        content: String,
        tags: String,
        decoration_color: String,
    ) -> Result<notes::Model, CoreErrors> {
        // Step 1: Create a new note as an active model
        let new_note = notes::ActiveModel {
            user_id: Set(user_id),       // Associate the note with the user
            company_id: Set(company_id), // Optionally associate with a company
            project_id: Set(project_id), // Optionally associate with a project
            task_id: Set(task_id),       // Optionally associate with a task
            subtask_id: Set(subtask_id), // Optionally associate with a subtask
            content: Set(content),       // Set the content of the note
            tags: Set(Some(tags)),       // Set the tags (optional)
            decoration_color: Set(Some(decoration_color)), // Set the decoration color (optional)
            ..Default::default()         // Use default values for other fields
        };

        // Step 2: Insert the new note into the database
        let inserted_note = new_note.insert(db).await?;

        // Step 3: Return the successfully created note model
        Ok(inserted_note)
    }

    /// Checks if a user has permission to access a specific note.
    ///
    /// This function uses a SQL query to verify if the user has direct ownership of the note
    /// or access via project, company, task, or subtask associations, provided their role level
    /// is sufficiently privileged.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `note_id` - The ID of the note to check access for.
    /// * `user_id` - The ID of the user whose permissions are being checked.
    ///
    /// # Returns
    /// * `Result<Option<notes::Model>, CoreErrors>` - Returns the note model if the user has access,
    /// or `None` if the user lacks permission or the note does not exist.
    ///
    /// # Errors
    /// * Returns `CoreErrors` for any database operation failures.
    pub async fn check_user_permission(
        db: &DbConn,
        note_id: i32,
        user_id: i32,
    ) -> Result<Option<notes::Model>, CoreErrors> {
        // SQL query to validate user permissions for the specified note
        let sql = r#"
        WITH note_data AS (
            SELECT 
                n.*
            FROM 
                notes n
            WHERE 
                n.id = $1
        ),
        access_check AS (
            SELECT
                ua.*
            FROM 
                user_access ua
            WHERE 
                ua.user_id = $2
                AND (
                    (ua.project_id = (SELECT project_id FROM note_data) AND ua.role_id <= 2)
                    OR (ua.company_id = (SELECT company_id FROM note_data) AND ua.role_id <= 2)
                    OR (ua.task_id = (SELECT task_id FROM note_data) AND ua.role_id <= 2)
                    OR (ua.subtask_id = (SELECT subtask_id FROM note_data) AND ua.role_id <= 2)
                )
        ),
        final_note AS (
            SELECT 
                n.*
            FROM 
                note_data n
            WHERE 
                n.user_id = $2
                OR EXISTS (
                    SELECT 1 
                    FROM access_check
                )
        )
        SELECT 
            id,
            user_id,
            company_id,
            project_id,
            task_id,
            subtask_id,
            content,
            tags,
            decoration_color,
            created_at
        FROM 
            final_note;
    "#;

        // Step 1: Prepare the SQL statement with parameters
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                note_id.into(), // $1 - Note ID
                user_id.into(), // $2 - User ID
            ],
        );

        // Step 2: Execute the query and fetch the result
        let note: Option<notes::Model> = notes::Entity::find().from_raw_sql(stmt).one(db).await?;

        // Step 3: Return the note if found, or None if the user lacks permission
        Ok(note)
    }

    /// Deletes a note from the database by its ID.
    ///
    /// This function checks if the note exists before attempting to delete it. If the note exists,
    /// it is deleted. If the note does not exist, an error is returned.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `note_id` - The ID of the note to be deleted.
    ///
    /// # Returns
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` if the note is successfully deleted,
    /// or an appropriate error if the note does not exist or the deletion fails.
    ///
    /// # Errors
    /// * Returns `CoreErrors::DatabaseServiceError` if the note does not exist.
    /// * Returns `CoreErrors` for any database operation failures.
    pub async fn delete_note(db: &DbConn, note_id: i32) -> Result<(), CoreErrors> {
        // Step 1: Attempt to find the note by its ID
        match notes::Entity::find_by_id(note_id).one(db).await? {
            Some(note) => {
                // Step 2: Convert the found note entity into an active model for deletion
                let active_note = note.into_active_model();

                // Step 3: Delete the note record from the database
                active_note.delete(db).await?;
            }
            None => {
                // Step 4: Return an error if the note does not exist
                return Err(CoreErrors::DatabaseServiceError(format!(
                    "Note with ID {} does not exist",
                    note_id
                )));
            }
        }

        // Step 5: Return success if the note was deleted successfully
        Ok(())
    }
}
