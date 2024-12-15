use core_database::queries::companies_query::CompaniesQuery;
use core_debugger::tracing::{event, Level};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        companies_service_server::CompaniesService, CompanyInfoResponse, CompanyUserInfoResponse,
        CreateCompanyRequest, CreateCompanyResponse, DeleteCompanyRequest, GetAllCompanyRequest,
        GetAllCompanyRespnonse, StatusResponse, UserCompanyModificationRequest,
    },
    middleware::{
        access_check::check_company_permission,
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
    /// Creates a new company with validated name and assigns the authenticated user as the creator.
    ///
    /// This function validates the provided company name, creates a new company record in the database,
    /// and assigns the authenticated user as part of the company. It then returns a response with the
    /// new company's details.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `CreateCompanyRequest`, which includes the company name, description, and contact information.
    ///
    /// # Returns
    ///
    /// * `Result<Response<CreateCompanyResponse>, Status>` - A response with the newly created company's information,
    ///   or a `Status` error if validation or creation fails.
    async fn create_company(
        &self,
        request: Request<CreateCompanyRequest>,
    ) -> Result<Response<CreateCompanyResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received create company request: {:?}", request);

        // Step 1: Extract user ID from auth token in request metadata for authentication
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the request to access its inner data
        let request = request.into_inner();

        // Step 2: Establish a database connection
        let conn = &self.connection;

        // Step 3: Validate company name using a composite validator with multiple checks
        let composite_validator = CompositValidator::new(vec![
            empty_validation,             // Ensure name is not empty
            min_symbols_validator_3,      // Ensure name has at least 3 characters
            max_symbols_validator_20,     // Ensure name has no more than 20 characters
            no_special_symbols_validator, // Ensure name contains no special symbols
        ]);

        // Run the validation and capture the validated name
        let validated_project_name = composite_validator.validate(request.name)?;

        // Step 4: Create a new company record in the database using the validated name
        let company = CompaniesQuery::create_new_company(
            conn,
            user_id_from_token as i32,
            validated_project_name,
            request.description,
            request.contact_info,
        )
        .await?;

        // Step 5: Construct a success response with the new company details
        let response = Response::new(CreateCompanyResponse {
            company_id: company.id,
            name: company.name,
            description: company.description,
            contact_info: company.contact_info,
        });

        event!(target: "hellai_app_core_events", Level::DEBUG, "Created company response: {:?}", response);

        Ok(response)
    }

    /// Adds a user to a company with a specified role, if the current user has the required permissions.
    ///
    /// This function verifies that the authenticated user has permission to add other users to a company.
    /// If permission is granted, it adds the specified user to the company and returns their information.
    /// Otherwise, it returns a permission denied error.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `UserCompanyModificationRequest`, which includes
    ///   the ID of the user to be added and the company ID.
    ///
    /// # Returns
    ///
    /// * `Result<Response<CompanyUserInfoResponse>, Status>` - A response with the user's company info
    ///   if successful, or a permission denied error if the user lacks authorization.
    async fn add_user_to_company(
        &self,
        request: Request<UserCompanyModificationRequest>,
    ) -> Result<Response<CompanyUserInfoResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received request: {:?}", request);

        // Step 1: Authenticate the user by extracting their ID from the auth token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the request to access the inner data
        let request = request.into_inner();

        // Step 2: Establish a database connection
        let conn = &self.connection;

        // Step 3: Check if the authenticated user has permission to modify the company's users
        let user_company_access =
            check_company_permission(conn, user_id_from_token as i32, request.company_id).await?;

        // Permission level check - allow access if the user's role is sufficiently privileged (role_id <= 2)
        if user_company_access.role_id <= 2 {
            // Step 4: Add the specified user to the company with the necessary permissions
            let user_company =
                CompaniesQuery::add_user_to_company(conn, request.user_id, request.company_id)
                    .await?;

            // Step 5: Prepare the response with the new user's role information
            let response = Response::new(CompanyUserInfoResponse {
                user_id: user_company.user_id,
                user_role: user_company.role_id - 1, // Adjust to match the gRPC enum by subtracting 1
            });

            event!(target: "hellai_app_core_events", Level::DEBUG, "Response: {:?}", response);
            return Ok(response);
        }

        // Log and return a permission denied error if the user lacks authorization
        event!(target: "hellai_app_core_events", Level::DEBUG, "permission_denied");
        Err(Status::permission_denied("permission_denied"))
    }

    /// Removes a specified user from a company if the authenticated user has the required permissions.
    ///
    /// This function checks if the authenticated user is authorized to remove the specified user from the company.
    /// If the user is authorized (either removing themselves or having a sufficiently privileged role),
    /// it proceeds with the removal and returns a success response. If unauthorized, it returns a permission error.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `UserCompanyModificationRequest`, which includes
    ///   the ID of the user to be removed and the company ID.
    ///
    /// # Returns
    ///
    /// * `Result<Response<StatusResponse>, Status>` - A response indicating success if the user was removed,
    ///   or a permission error if the authenticated user lacks the required authorization.
    async fn remove_user_from_company(
        &self,
        request: Request<UserCompanyModificationRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received remove user request: {:?}", request);

        // Step 1: Authenticate the user by extracting their ID from the auth token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the request to access the inner data
        let request = request.into_inner();

        // Establish a database connection
        let conn = &self.connection;

        // Step 2: Check if the authenticated user is trying to remove themselves
        if user_id_from_token as i32 == request.user_id {
            // User is removing themselves, proceed with the removal
            CompaniesQuery::remove_user_from_company(
                conn,
                request.user_id,
                request.company_id,
                None,
            )
            .await?;
        } else {
            // Step 3: Verify that the authenticated user has the required permissions to remove other users
            let user_company_access =
                check_company_permission(conn, user_id_from_token as i32, request.company_id)
                    .await?;

            // Permission level check - allow access if the user's role is sufficiently privileged (role_id <= 2)
            if user_company_access.role_id <= 2 {
                // Authorized to remove the specified user, proceed with removal
                CompaniesQuery::remove_user_from_company(
                    conn,
                    request.user_id,
                    request.company_id,
                    Some(user_company_access.role_id),
                )
                .await?;
            } else {
                // Log and return a permission denied error if the user lacks authorization
                event!(target: "hellai_app_core_events", Level::DEBUG, "Permission denied for removing user");
                return Err(Status::permission_denied("permission_denied"));
            }
        }

        // Step 4: Prepare a success response indicating the user was removed
        let response = Response::new(StatusResponse { success: true });

        event!(target: "hellai_app_core_events", Level::DEBUG, "Response: {:?}", response);
        Ok(response)
    }

    /// Deletes a specified company and all associated users if the authenticated user has the "Owner" role.
    ///
    /// This function checks if the authenticated user has the "Owner" role for the specified company.
    /// If authorized, it deletes the company and removes all associated users. If unauthorized, it returns
    /// a permission denied error.
    ///
    /// # Arguments
    ///
    /// * `request` - A gRPC request containing `DeleteCompanyRequest`, which includes the company ID to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<Response<StatusResponse>, Status>` - A success response if the company and associated users are deleted,
    ///   or a permission denied error if the user lacks the required authorization.
    async fn delete_company(
        &self,
        request: Request<DeleteCompanyRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received delete company request: {:?}", request);

        // Step 1: Authenticate the user and extract their ID from the auth token in the request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Unwrap the request to access the inner data
        let request = request.into_inner();

        // Step 2: Establish a database connection
        let conn = &self.connection;

        // Step 3: Verify if the authenticated user has the "Owner" role for the specified company

        let user_company_access =
            check_company_permission(conn, user_id_from_token as i32, request.company_id).await?;

        // Step 4: Proceed with deletion only if the user has the "Owner" role (role_id == 1)
        if user_company_access.role_id == 1 {
            // Delete the specified company
            CompaniesQuery::delete_company(conn, request.company_id).await?;
            // Delete all user associations with the specified company
            CompaniesQuery::delete_all_users_from_company(conn, request.company_id).await?;

            // Step 5: Construct a success response indicating successful deletion
            let response = Response::new(StatusResponse { success: true });

            event!(target: "hellai_app_core_events", Level::DEBUG, "Company and associated users deleted successfully. Response: {:?}", response);
            Ok(response)
        } else {
            // Log and return a permission denied error if the user lacks "Owner" role authorization
            event!(target: "hellai_app_core_events", Level::DEBUG, "Permission denied: User lacks 'Owner' role to delete company");
            Err(Status::permission_denied("permission_denied"))
        }
    }

    /// Retrieves all companies associated with a user.
    ///
    /// This function authenticates the user using the auth token in the request metadata
    /// and fetches all companies associated with their user ID from the database.
    ///
    /// # Arguments
    /// * `request` - A gRPC `Request` object.
    ///
    /// # Returns
    /// * `Result<Response<GetAllCompanyRespnonse>, Status>` - Returns a gRPC response containing a list of companies,
    /// or a gRPC `Status` error if authentication or database operations fail.
    ///
    /// # Errors
    /// * Returns `Status` for authentication errors or database failures.
    async fn get_all_user_companies(
        &self,
        request: Request<GetAllCompanyRequest>,
    ) -> Result<Response<GetAllCompanyRespnonse>, Status> {
        // Step 1: Log the incoming request
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Received get all user companies request: {:?}",
            request
        );

        // Step 2: Authenticate the user and extract their ID from the auth token
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Step 3: Establish a database connection
        let conn = &self.connection;

        // Step 4: Fetch all companies associated with the authenticated user
        let companies =
            CompaniesQuery::get_all_user_companies(conn, user_id_from_token as i32).await?;

        // Step 5: Transform the database results into the gRPC response format
        let companies_response: Vec<CompanyInfoResponse> = companies
            .into_iter()
            .map(|company| CompanyInfoResponse {
                id: company.id,
                name: company.name,
                name_alias: company.name_alias,
                description: company.description,
                contact_info: company.contact_info,
            })
            .collect();

        // Step 6: Construct the response object
        let response = Response::new(GetAllCompanyRespnonse {
            companies: companies_response,
        });

        // Step 7: Log the success event
        event!(
            target: "hellai_app_core_events",
            Level::DEBUG,
            "Retrieved all user companies successfully. Response: {:?}",
            response
        );

        Ok(response)
    }
}
