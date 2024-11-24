use core_error::core_errors::CoreErrors;
use sea_orm::{ActiveModelTrait, DbConn, Set};

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
}
