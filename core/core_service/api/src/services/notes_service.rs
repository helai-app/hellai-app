use core_database::queries::notes_query::NotesQuery;
use core_debugger::tracing::{event, Level};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        notes_service_server::NotesService, CreateNoteRequest, CreateNoteResponse,
        DeleteNoteRequest, NoteUserInfoResponse, StatusResponse, UserNoteModificationRequest,
    },
    middleware::{
        access_check::{
            check_company_permission, check_project_permission, check_tasks_permission,
        },
        interceptors,
        validators::{
            empty_validation, hex_color_validator, max_symbols_validator_20,
            max_symbols_validator_250, min_symbols_validator_3, no_special_symbols_validator,
            CompositValidator,
        },
    },
    my_server::MyServer,
};

#[tonic::async_trait]
impl NotesService for MyServer {
    /// Handles the creation of a new note with validation and permission checks.
    ///
    /// This function validates the incoming request, verifies the user's access permissions,
    /// and creates a new note in the database if all checks pass.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `CreateNoteRequest`, which includes note details and optional associations.
    ///
    /// # Returns
    ///
    /// * `Result<Response<CreateNoteResponse>, Status>` - Returns a success response with the created note details,
    ///   or an error if validation or permission checks fail.
    async fn create_note(
        &self,
        request: Request<CreateNoteRequest>,
    ) -> Result<Response<CreateNoteResponse>, Status> {
        // Log the incoming request at the DEBUG level
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Received create note request: {:?}",
            request
        );

        // Step 1: Authenticate the user by extracting their ID from the auth token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Step 2: Extract the inner request payload
        let request: CreateNoteRequest = request.into_inner();

        // Step 3: Validate the incoming request fields using composite validators
        let composite_validator_tags = CompositValidator::new(vec![
            empty_validation,
            min_symbols_validator_3,
            max_symbols_validator_20,
            no_special_symbols_validator,
        ]);

        let composite_validator_content = CompositValidator::new(vec![
            empty_validation,
            min_symbols_validator_3,
            max_symbols_validator_250,
        ]);

        let composite_validator_decoration_color =
            CompositValidator::new(vec![empty_validation, hex_color_validator]);

        // Validate individual fields and return an error if validation fails
        let validated_tags = composite_validator_tags.validate(request.tags)?;
        let validated_content = composite_validator_content.validate(request.content)?;
        let validated_decoration_color =
            composite_validator_decoration_color.validate(request.decoration_color)?;

        // Step 4: Establish a database connection
        let conn = &self.connection;

        // Step 5: Determine the user's access level based on the associated entity (company, project, or task)
        let access_lvl: Option<i32> = if let Some(company_id) = request.company_id {
            Some(
                check_company_permission(conn, user_id_from_token as i32, company_id)
                    .await?
                    .role_id,
            )
        } else if let Some(project_id) = request.project_id {
            Some(
                check_project_permission(conn, user_id_from_token as i32, project_id)
                    .await?
                    .user_role
                    .id,
            )
        } else if let Some(task_id) = request.task_id {
            Some(
                check_tasks_permission(conn, user_id_from_token as i32, task_id)
                    .await?
                    .1,
            )
        } else if request.subtask_id.is_some() {
            // Subtasks are not supported at this time
            return Err(Status::invalid_argument("bad_format"));
        } else {
            None // No associated entity; personal note
        };

        // Step 6: Permission check - allow access if the user's role is sufficiently privileged (role_id <= 2)
        if let Some(level) = access_lvl {
            if level > 2 {
                return Err(Status::permission_denied("permission_denied"));
            }
        }

        // Step 7: Create the note in the database
        let note = NotesQuery::create_note(
            conn,
            user_id_from_token as i32,
            request.project_id,
            request.company_id,
            request.task_id,
            request.subtask_id,
            validated_content,
            validated_tags,
            validated_decoration_color,
        )
        .await?;

        // Step 8: Prepare the response with the created note details
        let response = Response::new(CreateNoteResponse {
            note_id: note.id,
            content: note.content,
            tags: note.tags.unwrap_or_default(),
            decoration_color: note.decoration_color.unwrap_or_default(),
        });

        // Log the response at the DEBUG level
        event!(target: "hellai_app_core_events", Level::DEBUG, "Response: {:?}", response);

        Ok(response)
    }

    async fn add_user_to_note(
        &self,
        request: Request<UserNoteModificationRequest>,
    ) -> Result<Response<NoteUserInfoResponse>, Status> {
        todo!()
    }

    async fn remove_user_from_note(
        &self,
        request: Request<UserNoteModificationRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!()
    }

    async fn delete_note(
        &self,
        request: Request<DeleteNoteRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!()
    }
}
