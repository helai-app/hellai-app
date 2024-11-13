use core_debugger::tracing::{event, Level};
// use core_database::queries::projects_query::ProjectQuery;
// use core_debugger::tracing::{event, Level};
use sea_orm::DbConn;
use tonic::{Request, Response, Status};

// use crate::{
//     helai_api_core_service::{
//         projects_service_server::ProjectsService, CreateProjectRequest, CreateProjectResponse,
//         DeleteProjectRequest, ProjectUserInfoResponse, StatusResponse,
//         UserProjectModificationRequest,
//     },
//     middleware::{
//         interceptors,
//         validators::{
//             empty_validation, max_symbols_validator_20, min_symbols_validator_3,
//             no_special_symbols_validator, CompositValidator,
//         },
//     },
//     my_server::MyServer,
// };
use crate::{
    helai_api_core_service::{
        projects_service_server::ProjectsService, CreateProjectRequest, CreateProjectResponse,
        DeleteProjectRequest, ProjectUserInfoResponse, StatusResponse,
        UserProjectModificationRequest,
    },
    middleware::validators::{
        empty_validation, max_symbols_validator_20, min_symbols_validator_3,
        no_special_symbols_validator, CompositValidator,
    },
    my_server::MyServer,
};

// Implementing the ProjectsService trait for MyServer
#[tonic::async_trait]
impl ProjectsService for MyServer {
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

        // Unwrap the request to access its inner data
        let request = request.into_inner();

        // Validate project name using composite validator
        let composite_validator = CompositValidator::new(vec![
            empty_validation,
            min_symbols_validator_3,
            max_symbols_validator_20,
            no_special_symbols_validator,
        ]);

        let validated_project_name = composite_validator.validate(request.project_name)?;

        todo!()
    }

    async fn add_user_to_project(
        &self,
        request: Request<UserProjectModificationRequest>,
    ) -> Result<Response<ProjectUserInfoResponse>, Status> {
        todo!()
    }

    async fn remove_user_from_project(
        &self,
        request: Request<UserProjectModificationRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!()
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
//         _ => Err(Status::permission_denied("insufficient_rights")),
//     }
// }
