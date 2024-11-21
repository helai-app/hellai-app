use core_database::queries::tasks_query::TasksQuery;
use core_debugger::tracing::{event, Level};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        tasks_service_server::TasksService, CreateTaskRequest, CreateTaskResponse,
        DeleteTaskRequest, StatusResponse, TaskUserInfoResponse, UserTaskModificationRequest,
    },
    middleware::{
        access_check::{check_project_permission, check_tasks_permission},
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

    /// Adds a user to a task after verifying the permissions of the authenticated user.
    ///
    /// This function validates the authenticated user's permissions for the task, and if the permissions are sufficient,
    /// it adds the specified user to the task by creating or updating a `user_access` record.
    ///
    /// # Arguments
    /// * `request` - A gRPC `Request` object containing the user and task details for the operation.
    ///
    /// # Returns
    /// * `Result<Response<TaskUserInfoResponse>, Status>` - Returns a gRPC response containing the added user's information,
    /// or a gRPC `Status` error if validation or permissions checks fail.
    ///
    /// # Errors
    /// * Returns `Status::permission_denied` if the authenticated user lacks sufficient privileges.
    /// * Returns `Status` for any other errors encountered during processing.
    async fn add_user_to_task(
        &self,
        request: Request<UserTaskModificationRequest>,
    ) -> Result<Response<TaskUserInfoResponse>, Status> {
        // Log the incoming request at the DEBUG level
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Received add user to task request: {:?}",
            request
        );

        // Step 1: Authenticate the user by checking the auth token in the request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the gRPC request to access the inner payload
        let request = request.into_inner();

        // Step 2: Establish a connection to the database
        let conn = &self.connection;

        // Step 3: Verify the authenticated user's permissions for the specified task
        let user_task_access =
            check_tasks_permission(conn, user_id_from_token as i32, request.task_id).await?;

        // Allow access only if the user's role ID is sufficiently privileged (role_id <= 2)
        if user_task_access.1 <= 2 {
            // Step 4: Add the specified user to the task
            let user_access =
                TasksQuery::add_user_to_task(conn, request.user_id, request.task_id).await?;

            // Step 5: Construct a success response with the added user's details
            let response = Response::new(TaskUserInfoResponse {
                user_id: user_access.user_id,
                user_role: user_access.role_id.unwrap_or(0) - 1, // Adjust role ID to match the gRPC enum by subtracting 1
            });

            // Log the success event
            event!(
                target: "hellai_app_core_events",
                Level::DEBUG,
                "User added to task successfully. Response: {:?}",
                response
            );

            Ok(response)
        } else {
            // Step 6: Log and return a permission denied error if the user lacks sufficient privileges
            event!(
                target: "hellai_app_core_events",
                Level::DEBUG,
                "Permission denied: User lacks sufficient privileges to add users to task"
            );

            Err(Status::permission_denied(
                "Permission denied: insufficient privileges",
            ))
        }
    }

    /// Removes a user from a task after validating the permissions of the authenticated user.
    ///
    /// This function allows a user to remove themselves from a task or, if authorized, remove another user.
    /// It validates permissions and handles both cases appropriately.
    ///
    /// # Arguments
    /// * `request` - A gRPC `Request` object containing the user and task details for the operation.
    ///
    /// # Returns
    /// * `Result<Response<StatusResponse>, Status>` - Returns a gRPC response indicating success,
    /// or a gRPC `Status` error if validation or permissions checks fail.
    ///
    /// # Errors
    /// * Returns `Status::permission_denied` if the authenticated user lacks sufficient privileges.
    /// * Returns `Status` for any other errors encountered during processing.
    async fn remove_user_from_task(
        &self,
        request: Request<UserTaskModificationRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        // Step 1: Log the incoming request at the DEBUG level
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Received remove user from task request: {:?}",
            request
        );

        // Step 2: Authenticate the user by checking the auth token in the request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the gRPC request to access the inner payload
        let request = request.into_inner();

        // Step 3: Establish a connection to the database
        let conn = &self.connection;

        // Step 4: Handle user removal logic
        if user_id_from_token as i32 == request.user_id {
            // Case 1: User is removing themselves from the task
            TasksQuery::remove_user_from_task(conn, request.user_id, request.task_id, None).await?;
        } else {
            // Case 2: User is attempting to remove another user from the task

            // Check if the authenticated user has sufficient permissions for the task
            let user_task_access =
                check_tasks_permission(conn, user_id_from_token as i32, request.task_id).await?;

            println!("Permission id {}", user_task_access.1);

            if user_task_access.1 <= 2 {
                // User has sufficient privileges; proceed with the removal
                TasksQuery::remove_user_from_task(
                    conn,
                    request.user_id,
                    request.task_id,
                    Some(user_task_access.1),
                )
                .await?;
            } else {
                // Log and return a permission denied error
                event!(
                    target: "hellai_app_core_events",
                    Level::DEBUG,
                    "Permission denied: Insufficient role level for removing user"
                );
                return Err(Status::permission_denied(
                    "Permission denied: insufficient privileges",
                ));
            }
        }

        // Step 5: Construct a success response indicating the user was removed
        let response = Response::new(StatusResponse { success: true });

        // Log the success event
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "User removed from task successfully. Response: {:?}",
            response
        );

        Ok(response)
    }

    /// Deletes a task and all associated user-task relationships from the database.
    ///
    /// This function validates the authenticated user's permissions, ensuring only users with the "Owner" role
    /// (role_id == 1) can delete a task. Upon successful validation, the task and all associated user-task
    /// relationships are removed from the database.
    ///
    /// # Arguments
    /// * `request` - A gRPC `Request` object containing the task ID to be deleted.
    ///
    /// # Returns
    /// * `Result<Response<StatusResponse>, Status>` - Returns a gRPC response indicating success,
    /// or a gRPC `Status` error if the user lacks sufficient permissions or if any database operation fails.
    ///
    /// # Errors
    /// * Returns `Status::permission_denied` if the user lacks the required "Owner" role.
    /// * Returns `Status` for any other errors encountered during processing.
    async fn delete_task(
        &self,
        request: Request<DeleteTaskRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        // Log the incoming request
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Received delete task request: {:?}",
            request
        );

        // Step 1: Authenticate the user by checking the auth token in the request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Step 2: Unwrap the gRPC request to access the inner payload
        let request = request.into_inner();

        // Step 3: Establish a connection to the database
        let conn = &self.connection;

        // Step 4: Verify the authenticated user's permissions for the specified task
        let user_task_access =
            check_tasks_permission(conn, user_id_from_token as i32, request.task_id).await?;

        // Allow deletion only if the user's role is "Owner" (role_id == 1)
        if user_task_access.1 == 1 {
            // Step 5: Delete the task from the database
            TasksQuery::delete_task(conn, request.task_id).await?;

            // Step 6: Delete all user associations with the specified task
            TasksQuery::delete_all_users_from_task(conn, request.task_id).await?;

            // Step 7: Construct a success response
            let response = Response::new(StatusResponse { success: true });

            // Log the success event
            event!(
                target: "hellai_app_core_events",
                Level::DEBUG,
                "Task and associated users deleted successfully. Response: {:?}",
                response
            );

            Ok(response)
        } else {
            // Log and return a permission denied error if the user lacks the required "Owner" role
            event!(
                target: "hellai_app_core_events",
                Level::DEBUG,
                "Permission denied: User lacks 'Owner' role to delete task"
            );

            Err(Status::permission_denied(
                "Permission denied: insufficient privileges",
            ))
        }
    }
}
