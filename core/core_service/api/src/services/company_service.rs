use core_database::queries::companies_query::CompaniesQuery;
use core_debugger::tracing::{event, Level};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        companies_service_server::CompaniesService, CompanyUserInfoResponse, CreateCompanyRequest,
        CreateCompanyResponse, DeleteCompanyRequest, StatusResponse,
        UserCompanyModificationRequest,
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

// Implementing the CompaniesService trait for MyServer
#[tonic::async_trait]
impl CompaniesService for MyServer {
    async fn create_company(
        &self,
        request: Request<CreateCompanyRequest>,
    ) -> Result<Response<CreateCompanyResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the request to access its inner data
        let request = request.into_inner();
        // Extract user ID from auth token in request metadata

        // Validate project name using composite validator
        let composite_validator = CompositValidator::new(vec![
            empty_validation,
            min_symbols_validator_3,
            max_symbols_validator_20,
            no_special_symbols_validator,
        ]);

        let validated_project_name = composite_validator.validate(request.name)?;

        // Extract database connection
        let conn = &self.connection;

        let company = CompaniesQuery::create_new_company(
            conn,
            user_id_from_token as i32,
            validated_project_name,
            request.description,
            request.contact_info,
        )
        .await?;

        // Create success response
        let response = Response::new(CreateCompanyResponse {
            company_id: company.id,
            name: company.name,
            description: company.description,
            contact_info: company.contact_info,
        });

        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);

        Ok(response)
    }

    async fn add_user_to_company(
        &self,
        request: Request<UserCompanyModificationRequest>,
    ) -> Result<Response<CompanyUserInfoResponse>, Status> {
        todo!();
    }

    async fn remove_user_from_company(
        &self,
        request: Request<UserCompanyModificationRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!();
    }

    async fn delete_company(
        &self,
        request: Request<DeleteCompanyRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!();
    }
}
