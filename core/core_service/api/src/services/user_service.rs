use core_database::queries::{companies_query::CompaniesQuery, user_query::UserQuery};
use core_debugger::tracing::{event, Level};
use helai_api_core_service::{
    user_service_server::UserService, AuthUserCompanyProjectsInfoResponse,
    AuthenticateWithPasswordRequest, RefreshSessionTokenRequest, RegisterUserRequest,
    TokenResponse,
};

use middleware::auth_token::{RefreshClaims, SessionClaims};

use service::password_validation::{hash_password, verify_hash_password};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{self, *},
    middleware::{self, *},
    my_server::MyServer,
};

#[tonic::async_trait]
impl UserService for MyServer {
    /// 🧨 Get user data with creds.
    async fn authenticate_with_password(
        &self,
        request: Request<AuthenticateWithPasswordRequest>,
    ) -> Result<Response<AuthUserCompanyProjectsInfoResponse>, Status> {
        // Log incoming request for authentication debugging
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received authentication request: {:?}", request);

        // Extract connection for database queries
        let conn = &self.connection;
        let request = request.into_inner();

        // Validate login and password formats using custom validators
        let login = validators::login_format_validation(request.login)?;
        let password = validators::password_format_validation(request.password)?;

        // Retrieve user information by login from the database
        let user_with_password = match UserQuery::get_user_by_login(conn, login).await? {
            Some(user) => user,
            None => {
                return Err(Status::invalid_argument(
                    "User not found: Invalid login credentials",
                ));
            }
        };

        let user = user_with_password.0;

        // Verify the provided password against the stored password hash
        if !verify_hash_password(&user_with_password.1.password_hash, &password)? {
            return Err(Status::invalid_argument(
                "Authentication failed: Incorrect password",
            ));
        }

        // Generate session and refresh tokens for the authenticated user
        let session_claims = SessionClaims::new(user.id as i64);
        let session_token = session_claims.into_token()?;

        let refresh_claims = RefreshClaims::new(user.id as i64);
        let refresh_token = refresh_claims.into_token()?;

        // Fetch user's associated company and project information
        let user_company_with_projects =
            CompaniesQuery::get_company_with_projects(conn, user.id, None).await?;

        // Format response structure for company and projects, if available
        let (company_info, projects) = match user_company_with_projects {
            Some(company) => (
                Some(CompanyInfoResponse {
                    id: company.id,
                    name: company.name,
                    name_alias: company.name_alias,
                    description: company.description,
                    contact_info: company.contact_info,
                }),
                company
                    .company_projects
                    .into_iter()
                    .map(|project| ProjectsResponse {
                        id: project.id,
                        company_id: project.company_id,
                        title: project.title,
                        description: project.description,
                        decoration_color: project.decoration_color,
                    })
                    .collect(),
            ),
            None => (None, vec![]), // Empty vector if no projects are found
        };

        // Construct response with user and company/project details
        let reply = AuthUserCompanyProjectsInfoResponse {
            user_id: user.id,
            email: user.email,
            user_name: user.user_name,
            login: user.login,
            session_token,
            refresh_token,
            company: company_info,
            user_projects: projects,
        };

        // Wrap response in gRPC Response object and log it
        let response = Response::new(reply);
        event!(target: "hellai_app_core_events", Level::DEBUG, "Authentication response: {:?}", response);

        // Return successful response
        Ok(response)
    }

    /// 🧨 Get user data with auth token.
    async fn get_user_data(
        &self,
        request: Request<GetUserDataRequest>,
    ) -> Result<Response<UserCompanyProjectsInfoResponse>, Status> {
        // Log incoming request for debugging
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received user data request: {:?}", request);

        // Extract database connection
        let conn = &self.connection;

        // Extract user ID from authentication token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Retrieve user information by user ID from the database
        let user = match UserQuery::get_user_by_id(conn, user_id_from_token as i32).await? {
            Some(user) => user,
            None => return Err(Status::invalid_argument("User not found: Invalid user ID")),
        };

        // Fetch user's associated company and project information
        let user_company_with_projects =
            CompaniesQuery::get_company_with_projects(conn, user.id, None).await?;

        // Format response structure for company and projects, if available
        let (company_info, projects) = match user_company_with_projects {
            Some(company) => (
                Some(CompanyInfoResponse {
                    id: company.id,
                    name: company.name,
                    name_alias: company.name_alias,
                    description: company.description,
                    contact_info: company.contact_info,
                }),
                company
                    .company_projects
                    .into_iter()
                    .map(|project| ProjectsResponse {
                        id: project.id,
                        company_id: project.company_id,
                        title: project.title,
                        description: project.description,
                        decoration_color: project.decoration_color,
                    })
                    .collect(),
            ),
            None => (None, vec![]), // Return an empty vector if no projects are found
        };

        // Construct response with user and company/project details
        let reply = UserCompanyProjectsInfoResponse {
            user_id: user.id,
            email: user.email,
            user_name: user.user_name,
            login: user.login,
            company: company_info,
            user_projects: projects,
        };

        // Wrap response in gRPC Response object and log it
        let response = Response::new(reply);
        event!(target: "hellai_app_core_events", Level::DEBUG, "User data response: {:?}", response);

        // Return successful response
        Ok(response)
    }

    /// 🧨 Register new user.
    async fn register_user(
        &self,
        request: Request<RegisterUserRequest>,
    ) -> Result<Response<NewUserResponse>, Status> {
        // Log the incoming registration request for debugging purposes
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received registration request: {:?}", request);

        // Extract the database connection
        let conn = &self.connection;

        // Unwrap the request to access its inner data
        let request = request.into_inner();

        // Validate input data formats
        let login = validators::login_format_validation(request.login)?;
        let user_name = validators::login_format_validation(request.user_name)?;
        let password = validators::password_format_validation(request.password)?;
        let email = validators::email_format_validation(request.email)?;

        // Hash the password for secure storage
        let hashed_password = hash_password(password.as_str())?;

        // Create a new user record in the database
        let new_user =
            UserQuery::create_new_user(conn, login, user_name, hashed_password.0, email).await?;

        // Log the newly created user details
        event!(target: "hellai_app_core_events", Level::DEBUG, "New user created: {:?}", new_user);

        // Generate session and refresh tokens for the new user
        let session_claims = SessionClaims::new(new_user.id as i64);
        let session_token = session_claims.into_token()?;

        let refresh_claims = RefreshClaims::new(new_user.id as i64);
        let refresh_token = refresh_claims.into_token()?;

        // Prepare the response with new user details and tokens
        let reply = NewUserResponse {
            user_id: new_user.id,
            email: new_user.email,
            login: new_user.login,
            user_name: new_user.user_name,
            session_token,
            refresh_token,
        };

        // Wrap the response in a gRPC Response object and log it
        let response = Response::new(reply);
        event!(target: "hellai_app_core_events", Level::DEBUG, "Registration response: {:?}", response);

        // Return the successful response
        Ok(response)
    }

    /// 🧨 Refreshes the user's session token using a provided refresh token.
    async fn refresh_session_token(
        &self,
        request: Request<RefreshSessionTokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        // Log the incoming request for debugging purposes
        event!(target: "hellai_app_core_events", Level::DEBUG, "Received refresh token request: {:?}", request);

        // Extract the inner data from the gRPC request
        let request = request.into_inner();

        // Extract the refresh token from the request
        let refresh_token = request.refresh_token;

        // Decode and validate the refresh token claims
        let refresh_claims = RefreshClaims::from_token(refresh_token)?;

        // Generate a new session token using the user ID from the refresh claims
        let session_claims = SessionClaims::new(refresh_claims.sub);
        let session_token = session_claims.into_token()?;

        // Prepare the response containing the new session token
        let reply = TokenResponse { session_token };
        let response = Response::new(reply);

        // Log the response for debugging purposes
        event!(target: "hellai_app_core_events", Level::DEBUG, "Generated new session token response: {:?}", response);

        // Return the successful response
        Ok(response)
    }
}
