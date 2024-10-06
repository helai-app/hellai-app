use core_database::queries::projects_query::ProjectQuery;
use core_debugger::tracing::{event, Level};
use sea_orm::DbConn;
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        projects_service_server::ProjectsService, CreateProjectRequest, CreateProjectResponse,
        DeleteProjectRequest, ProjectUserInfoResponse, StatusResponse,
        UserProjectModificationRequest,
    },
    middleware::{
        interceptors,
        validators::{
            empty_validation, max_symbols_validator_20, min_symbols_validator_3,
            no_special_symbols_validator, CompositValidator,
        },
    },
    my_server::MyServer,
};

#[tonic::async_trait]
impl ProjectsService for MyServer {
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

        // Check if user auth and get his id
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        let conn = &self.connection;

        let request = request.into_inner();

        // Validate
        let composite_validator = CompositValidator::new(vec![
            empty_validation,
            min_symbols_validator_3,
            max_symbols_validator_20,
            no_special_symbols_validator,
        ]);

        let result = composite_validator.validate(request.project_name)?;

        let new_project =
            ProjectQuery::create_project(conn, result, user_id_from_token as i32).await?;

        // Create respnse
        let reply = CreateProjectResponse {
            project_id: new_project.id,
            project_name: new_project.name,
        };

        let response = Response::new(reply);

        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);
        Ok(response)
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);
        // Check if user auth and get his id
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        let conn = &self.connection;

        let request = request.into_inner();

        check_project_permision(conn, user_id_from_token as i32, request.project_id).await?;

        ProjectQuery::delete_project(conn, request.project_id).await?;

        let response = Response::new(StatusResponse { success: true });

        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);

        Ok(response)
    }

    async fn add_user_to_project(
        &self,
        request: Request<UserProjectModificationRequest>,
    ) -> Result<Response<ProjectUserInfoResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);
        // Check if user auth and get his id
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        let conn = &self.connection;

        let request = request.into_inner();

        let admin_id = user_id_from_token as i32;
        let project_id = request.project_id;
        let user_id = request.user_id;

        check_project_permision(conn, admin_id, project_id).await?;

        let user_role = ProjectQuery::set_user_project_role(conn, user_id, project_id, 3).await?;

        let response = Response::new(ProjectUserInfoResponse {
            user_id: user_id,
            user_role: user_role.project_role_id - 1,
        });

        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);

        Ok(response)
    }

    async fn remove_user_from_project(
        &self,
        request: Request<UserProjectModificationRequest>,
    ) -> Result<Response<ProjectUserInfoResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
        println!("user_id_from_token: {}", user_id_from_token);
        todo!()
    }
}

// Check that user have permision to project
async fn check_project_permision(
    conn: &DbConn,
    user_id: i32,
    project_id: i32,
) -> Result<(), Status> {
    // Check that user have permision Owner lvl
    let user_role_in_project =
        ProjectQuery::get_user_role_in_project(conn, project_id, user_id).await?;

    match user_role_in_project {
        Some(user_role_in_project) => {
            if user_role_in_project.project_role_id != 1 {
                return Err(Status::permission_denied("insufficient_rights"));
            }
        }
        None => return Err(Status::permission_denied("insufficient_rights")),
    }

    Ok(())
}
