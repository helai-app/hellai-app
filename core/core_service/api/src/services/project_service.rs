use core_database::queries::projects_query::ProjectQuery;
use core_debugger::tracing::{event, Level};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        projects_service_server::ProjectsService, CreateProjectRequest, CreateProjectResponse,
        DeleteProjectRequest, GetAllCompanyProjectsRequest, GetAllCompanyProjectsRespnonse,
        ProjectUserInfoResponse, ProjectsResponse, StatusResponse, UserProjectModificationRequest,
    },
    middleware::{
        access_check::{check_company_permission, check_project_permission},
        interceptors,
        validators::{
            empty_validation, hex_color_validator, max_symbols_validator_20,
            max_symbols_validator_250, min_symbols_validator_3, no_special_symbols_validator,
            CompositValidator,
        },
    },
    my_server::MyServer,
};

// Implementing the ProjectsService trait for MyServer
#[tonic::async_trait]
impl ProjectsService for MyServer {
    /// Handles the creation of a new project within a specified company.
    ///
    /// This function validates the provided project details, checks if the user has sufficient permissions,
    /// and creates the project in the database if authorized.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `CreateProjectRequest`, which includes the project details and company ID.
    ///
    /// # Returns
    ///
    /// * `Result<Response<CreateProjectResponse>, Status>` - A response containing the newly created project's details,
    ///   or a permission denied error if the user lacks authorization.
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received create project request: {:?}", request);

        // Step 1: Authenticate the user by extracting their ID from the auth token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the request to access the inner data
        let request = request.into_inner();

        // Step 2: Validate the project details using composite validators
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

        let composite_validator_hex_color =
            CompositValidator::new(vec![empty_validation, hex_color_validator]);

        // Validate each field and handle errors if any
        let validated_project_title = composite_validator_title.validate(request.title)?;
        let validated_project_description =
            composite_validator_description.validate(request.description)?;
        let validated_project_decoration_color =
            composite_validator_hex_color.validate(request.decoration_color)?;

        // Step 3: Establish a database connection
        let conn = &self.connection;

        // Step 4: Check if the authenticated user has sufficient permissions for the specified company
        let user_company_access =
            check_company_permission(conn, user_id_from_token as i32, request.company_id).await?;

        // Permission level check - allow access if the user's role is sufficiently privileged (role_id <= 3)
        if user_company_access.role_id <= 3 {
            // Step 5: Create a new project in the database
            let new_project = ProjectQuery::create_project(
                conn,
                request.company_id,
                validated_project_title,
                validated_project_description,
                validated_project_decoration_color,
                user_id_from_token as i32,
                user_company_access.role_id,
            )
            .await?;

            // Step 6: Construct a success response with the new project details
            let response = Response::new(CreateProjectResponse {
                project_id: new_project.id,
                company_id: new_project.company_id,
                title: new_project.title,
                description: new_project.description.unwrap_or_default(),
                decoration_color: new_project.decoration_color.unwrap_or_default(),
            });

            event!(target: "hellai_app_core_events", Level::DEBUG, "Project created successfully. Response: {:?}", response);
            Ok(response)
        } else {
            // Log and return a permission denied error if the user lacks sufficient privileges
            event!(target: "hellai_app_core_events", Level::DEBUG, "Permission denied: User lacks sufficient privileges to create project");
            Err(Status::permission_denied("permission_denied"))
        }
    }

    /// Adds a user to a project if the authenticated user has sufficient permissions.
    ///
    /// This function checks if the authenticated user has a role with sufficient privileges in the project.
    /// If authorized, it adds the specified user to the project and returns a success response with the user's role.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `UserProjectModificationRequest`, which includes
    ///   the user ID and project ID to modify.
    ///
    /// # Returns
    ///
    /// * `Result<Response<ProjectUserInfoResponse>, Status>` - A response containing the added user's information,
    ///   or a permission denied error if the user lacks sufficient privileges.
    async fn add_user_to_project(
        &self,
        request: Request<UserProjectModificationRequest>,
    ) -> Result<Response<ProjectUserInfoResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received add user to project request: {:?}", request);

        // Step 1: Authenticate the user by extracting their ID from the auth token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the request to access the inner data
        let request = request.into_inner();

        // Step 2: Establish a database connection
        let conn = &self.connection;

        // Step 3: Check if the authenticated user has sufficient permissions for the specified project
        let user_company_access =
            check_project_permission(conn, user_id_from_token as i32, request.project_id).await?;

        // Permission level check - allow access if the user's role is sufficiently privileged (role_id <= 2)
        if user_company_access.user_role.id <= 2 {
            // Step 4: Add the specified user to the project
            let user_access =
                ProjectQuery::add_user_to_project(conn, request.user_id, request.project_id)
                    .await?;

            // Step 5: Construct a success response with the added user's information
            let response = Response::new(ProjectUserInfoResponse {
                user_id: user_access.user_id,
                user_role: user_access.role_id.unwrap_or(0) - 1, // Adjust to match the gRPC enum by subtracting 1
            });

            event!(target: "hellai_app_core_events", Level::DEBUG, "User added to project successfully. Response: {:?}", response);
            Ok(response)
        } else {
            // Log and return a permission denied error if the user's role is not sufficiently privileged
            event!(target: "hellai_app_core_events", Level::DEBUG, "Permission denied: User lacks sufficient role level to add users to project");
            Err(Status::permission_denied("permission_denied"))
        }
    }

    /// Removes a user from a project if the authenticated user has sufficient permissions.
    ///
    /// This function allows a user to remove themselves from a project or remove another user if
    /// they have the required permissions. If unauthorized, it returns a permission denied error.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `UserProjectModificationRequest`, which includes
    ///   the user ID and project ID for the removal action.
    ///
    /// # Returns
    ///
    /// * `Result<Response<StatusResponse>, Status>` - Returns a success response if the user is removed,
    ///   or a permission denied error if the user lacks authorization.
    async fn remove_user_from_project(
        &self,
        request: Request<UserProjectModificationRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received remove user from project request: {:?}", request);

        // Step 1: Authenticate the user by extracting their ID from the auth token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the request to access the inner data
        let request = request.into_inner();

        // Step 2: Establish a database connection
        let conn = &self.connection;

        // Step 3: Allow the user to remove themselves from the project
        if user_id_from_token as i32 == request.user_id {
            // User is removing themselves; proceed with the removal
            ProjectQuery::remove_user_from_project(conn, request.user_id, request.project_id, None)
                .await?;
        } else {
            // Step 4: Check if the authenticated user has sufficient permissions to remove another user
            let user_company_access =
                check_project_permission(conn, user_id_from_token as i32, request.project_id)
                    .await?;

            // Permission level check - allow access if the user's role is sufficiently privileged (role_id <= 2)
            if user_company_access.user_role.id <= 2 {
                // Authorized to remove the specified user; proceed with the removal
                ProjectQuery::remove_user_from_project(
                    conn,
                    request.user_id,
                    request.project_id,
                    Some(user_company_access.user_role.id),
                )
                .await?;
            } else {
                // Log and return a permission denied error if the user lacks authorization
                event!(target: "hellai_app_core_events", Level::DEBUG, "Permission denied: Insufficient role level for removing user");
                return Err(Status::permission_denied("permission_denied"));
            }
        }

        // Step 5: Construct a success response indicating the user was removed
        let response = Response::new(StatusResponse { success: true });

        event!(target: "hellai_app_core_events", Level::DEBUG, "User removed from project successfully. Response: {:?}", response);
        Ok(response)
    }

    /// Deletes a specified project and all associated user roles if the authenticated user has the "Owner" role.
    ///
    /// This function verifies if the authenticated user has the "Owner" role for the project.
    /// If authorized, it deletes the project and all user associations with the project.
    /// If unauthorized, it returns a permission denied error.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `DeleteProjectRequest`, which includes the project ID to delete.
    ///
    /// # Returns
    ///
    /// * `Result<Response<StatusResponse>, Status>` - A success response if the project and associated users are deleted,
    ///   or a permission denied error if the user lacks sufficient privileges.
    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received delete project request: {:?}", request);

        // Step 1: Authenticate the user by extracting their ID from the auth token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Step 2: Unwrap the request to access its inner data
        let request = request.into_inner();

        // Step 3: Establish a database connection
        let conn = &self.connection;

        // Step 4: Verify if the user has sufficient permissions to delete the project
        let user_company_access =
            check_project_permission(conn, user_id_from_token as i32, request.project_id).await?;

        // Permission level check - allow deletion only if the user's role is "Owner" (role_id == 1)
        if user_company_access.user_role.id == 1 {
            // Step 5: Delete the specified project
            ProjectQuery::delete_project(conn, request.project_id).await?;

            // Step 6: Delete all user associations with the specified project
            ProjectQuery::delete_all_users_from_project(conn, request.project_id).await?;

            // Step 7: Construct and return a success response
            let response = Response::new(StatusResponse { success: true });

            event!(target: "hellai_app_core_events", Level::DEBUG, "Project and associated users deleted successfully. Response: {:?}", response);
            Ok(response)
        } else {
            // Log and return a permission denied error if the user lacks the required "Owner" role
            event!(target: "hellai_app_core_events", Level::DEBUG, "Permission denied: User lacks 'Owner' role to delete project");
            Err(Status::permission_denied("permission_denied"))
        }
    }

    /// Retrieves all company projects filtered by user access.
    ///
    /// This function authenticates the user, validates their access to the specified company,
    /// and fetches the projects the user has permissions to view or access.
    ///
    /// # Arguments
    /// * `request` - A gRPC `Request` containing the company ID.
    ///
    /// # Returns
    /// * `Result<Response<GetAllCompanyProjectsRespnonse>, Status>` - Returns a gRPC response containing
    ///   the list of projects, or a `Status` error if authentication or database operations fail.
    ///
    /// # Errors
    /// * Returns `Status` for authentication errors or database failures.
    async fn get_all_company_projects(
        &self,
        request: Request<GetAllCompanyProjectsRequest>,
    ) -> Result<Response<GetAllCompanyProjectsRespnonse>, Status> {
        // Step 1: Log the incoming request
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Received get company projects request: {:?}",
            request
        );

        // Step 2: Authenticate the user by extracting their ID from the auth token in the request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Step 3: Extract the inner payload from the gRPC request
        let request = request.into_inner();

        // Step 4: Establish a database connection
        let conn = &self.connection;

        // Step 5: Fetch projects accessible by the user for the specified company
        let projects_db = ProjectQuery::get_all_company_project_by_access(
            conn,
            request.company_id,
            user_id_from_token as i32,
        )
        .await?;

        // Step 6: Transform the database results into the gRPC response format
        let projects_response: Vec<ProjectsResponse> = projects_db
            .into_iter()
            .map(|project| ProjectsResponse {
                id: project.id,
                company_id: project.company_id,
                title: project.title,
                description: project.description,
                decoration_color: project.decoration_color,
            })
            .collect();

        // Step 7: Construct the response object
        let response = Response::new(GetAllCompanyProjectsRespnonse {
            projects: projects_response,
        });

        // Step 8: Log the success event
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Retrieved company projects successfully. Response: {:?}",
            response
        );

        Ok(response)
    }
}
