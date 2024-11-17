use core_database::queries::projects_query::ProjectQuery;
use core_debugger::tracing::{event, Level};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        projects_service_server::ProjectsService, CreateProjectRequest, CreateProjectResponse,
        DeleteProjectRequest, ProjectUserInfoResponse, StatusResponse,
        UserProjectModificationRequest,
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

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!()
    }
}
//     // Handle creating a new project
//     async fn create_project(
//         &self,
//         request: Request<CreateProjectRequest>,
//     ) -> Result<Response<CreateProjectResponse>, Status> {
//         event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

//         // Extract user ID from auth token in request metadata
//         let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
//         let conn = &self.connection;
//         let request = request.into_inner();

//         // Validate project name using composite validator
//         let composite_validator = CompositValidator::new(vec![
//             empty_validation,
//             min_symbols_validator_3,
//             max_symbols_validator_20,
//             no_special_symbols_validator,
//         ]);
//         let validated_project_name = composite_validator.validate(request.project_name)?;

//         // Create a new project in the database
//         let new_project =
//             ProjectQuery::create_project(conn, validated_project_name, user_id_from_token as i32)
//                 .await?;

//         // Create response with project details
//         let reply = CreateProjectResponse {
//             project_id: new_project.id,
//             project_name: new_project.name,
//         };
//         let response = Response::new(reply);

//         event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);
//         Ok(response)
//     }

//     // Handle deleting an existing project
//     async fn delete_project(
//         &self,
//         request: Request<DeleteProjectRequest>,
//     ) -> Result<Response<StatusResponse>, Status> {
//         event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

//         // Extract user ID from auth token in request metadata
//         let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
//         let conn = &self.connection;
//         let request = request.into_inner();

//         // Check if the user has permission to delete the project
//         check_project_permission(conn, user_id_from_token as i32, request.project_id).await?;

//         // Delete the project
//         ProjectQuery::delete_project(conn, request.project_id).await?;

//         // Create success response
//         let response = Response::new(StatusResponse { success: true });

//         event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);
//         Ok(response)
//     }

//     // Handle adding a user to a project
//     async fn add_user_to_project(
//         &self,
//         request: Request<UserProjectModificationRequest>,
//     ) -> Result<Response<ProjectUserInfoResponse>, Status> {
//         event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

//         // Extract user ID from auth token in request metadata
//         let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
//         let conn = &self.connection;
//         let request = request.into_inner();

//         let admin_id = user_id_from_token as i32;

//         // Check if the current user has permission to add users to the project
//         check_project_permission(conn, admin_id, request.project_id).await?;

//         // Assign a role to the new user in the project (role ID 3 represents the assigned role)
//         let user_role =
//             ProjectQuery::set_user_project_role(conn, request.user_id, request.project_id, 3)
//                 .await?;

//         // Create response with updated user information
//         let response = Response::new(ProjectUserInfoResponse {
//             user_id: request.user_id,
//             user_role: user_role.project_role_id - 1,
//         });

//         event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);
//         Ok(response)
//     }

//     // Handle removing a user from a project
//     async fn remove_user_from_project(
//         &self,
//         request: Request<UserProjectModificationRequest>,
//     ) -> Result<Response<StatusResponse>, Status> {
//         event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

//         // Extract user ID from auth token in request metadata
//         let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
//         let conn = &self.connection;
//         let request = request.into_inner();

//         let admin_id = user_id_from_token as i32;

//         // Check if the current user has permission to remove users from the project
//         check_project_permission(conn, admin_id, request.project_id).await?;

//         // Remove the user from the project
//         ProjectQuery::remove_user_from_project(conn, request.project_id, request.user_id).await?;

//         // Create success response
//         let response = Response::new(StatusResponse { success: true });

//         event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);
//         Ok(response)
//     }
// }

// // Function to check if the user has permission to modify the project
// async fn check_project_permission(
//     conn: &DbConn,
//     user_id: i32,
//     project_id: i32,
// ) -> Result<(), Status> {
//     // Retrieve the user's role in the project from the database
//     let user_role_in_project =
//         ProjectQuery::get_user_role_in_project(conn, project_id, user_id).await?;

//     // Ensure that the user has "Owner" level permission (project_role_id == 1)
//     match user_role_in_project {
//         Some(user_role) if user_role.project_role_id == 1 => Ok(()),
//         _ => Err(Status::permission_denied("permission_denied")),
//     }
// }
