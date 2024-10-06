use core_debugger::tracing::{event, Level};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        projects_service_server::ProjectsService, CreateProjectRequest, CreateProjectResponse,
        DeleteProjectRequest, ProjectUsersResponse, StatusResponse, UserProjectModificationRequest,
    },
    middleware::interceptors,
    my_server::MyServer,
};

#[tonic::async_trait]
impl ProjectsService for MyServer {
    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
        println!("user_id_from_token: {}", user_id_from_token);
        todo!()
    }

    async fn add_user_to_project(
        &self,
        request: Request<UserProjectModificationRequest>,
    ) -> Result<Response<ProjectUsersResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
        println!("user_id_from_token: {}", user_id_from_token);
        todo!()
    }

    async fn delete_project(
        &self,
        request: Request<DeleteProjectRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
        println!("user_id_from_token: {}", user_id_from_token);
        todo!()
    }

    async fn remove_user_from_project(
        &self,
        request: Request<UserProjectModificationRequest>,
    ) -> Result<Response<ProjectUsersResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;
        println!("user_id_from_token: {}", user_id_from_token);
        todo!()
    }
}
