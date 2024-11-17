use core_database::queries::tasks_query::TasksQuery;
use core_debugger::tracing::{event, Level};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        tasks_service_server::TasksService, CreateTaskRequest, CreateTaskResponse,
        DeleteTaskRequest, StatusResponse, TaskUserInfoResponse, UserTaskModificationRequest,
    },
    middleware::{
        access_check::check_project_permission,
        interceptors,
        validators::{
            empty_validation, max_symbols_validator_20, max_symbols_validator_250,
            min_symbols_validator_3, no_special_symbols_validator, CompositValidator,
        },
    },
    my_server::MyServer,
};

// Implementing the ProjectsService trait for MyServer
#[tonic::async_trait]
impl TasksService for MyServer {
    /// Handles the creation of a new task by validating inputs, checking permissions, and interacting with the database.
    ///
    /// # Arguments
    /// * `request` - A `Request` object containing the details of the task to be created.
    ///
    /// # Returns
    /// * `Result<Response<CreateTaskResponse>, Status>` - A gRPC response containing the newly created task details on success,
    /// or a gRPC `Status` error if validation, permission checks, or database interactions fail.
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        // Log the incoming request at the DEBUG level
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Received create task request: {:?}",
            request
        );

        // Step 1: Authenticate the user using the auth token from the request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Extract the inner request payload
        let request = request.into_inner();

        // Step 2: Validate the task details using composite validators for title and description
        let composite_validator_title = CompositValidator::new(vec![
            empty_validation,
            min_symbols_validator_3,
            max_symbols_validator_20,
            no_special_symbols_validator,
        ]);

        let composite_validator_description = CompositValidator::new(vec![
            empty_validation,
            min_symbols_validator_3,
            max_symbols_validator_250,
        ]);

        // Validate the task title and description, returning an error if validation fails
        let validated_task_title = composite_validator_title.validate(request.title)?;
        let validated_task_description =
            composite_validator_description.validate(request.description)?;

        // Step 3: Establish a database connection
        let conn = &self.connection;

        // Step 4: Check the user's permissions for the specified project
        let user_company_access =
            check_project_permission(conn, user_id_from_token as i32, request.project_id).await?;

        // Verify if the user has sufficient privileges (role_id <= 3 indicates privileged roles)
        if user_company_access.user_role.id <= 3 {
            // Step 5: Create a new task in the database
            let new_task = TasksQuery::create_task(
                conn,
                request.project_id,
                validated_task_title,
                validated_task_description,
                user_id_from_token as i32,
            )
            .await?;

            // Step 6: Construct and return a success response with the new task details
            let response = Response::new(CreateTaskResponse {
                task_id: new_task.id,
                project_id: new_task.project_id,
                title: new_task.title,
                description: new_task.description.unwrap_or_default(),
            });

            // Log the success event
            event!(
                target: "hellai_app_core_events",
                Level::DEBUG,
                "Task created successfully. Response: {:?}",
                response
            );

            Ok(response)
        } else {
            // Log and return a permission denied error if the user lacks sufficient privileges
            event!(
                target: "hellai_app_core_events",
                Level::DEBUG,
                "Permission denied: User lacks sufficient privileges to create task"
            );

            Err(Status::permission_denied(
                "Permission denied: insufficient privileges",
            ))
        }
    }

    async fn add_user_to_task(
        &self,
        request: Request<UserTaskModificationRequest>,
    ) -> Result<Response<TaskUserInfoResponse>, Status> {
        todo!()
    }

    async fn remove_user_from_task(
        &self,
        request: Request<UserTaskModificationRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!()
    }

    async fn delete_task(
        &self,
        request: Request<DeleteTaskRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!()
    }
}
