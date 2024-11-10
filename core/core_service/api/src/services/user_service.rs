use core_database::queries::{
    companies_query::{CompaniesQuery, UserCompany},
    projects_query::{ProjectQuery, UserProject},
    user_query::UserQuery,
};
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

    async fn get_user_data(
        &self,
        request: Request<EmptyGetUserRequest>,
    ) -> Result<Response<ClearUserResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

        let conn = &self.connection;

        // Extract user ID from auth token in request metadata
        let user_id_from_token = interceptors::check_auth_token(request.metadata())?;

        // Get user info from bd
        let user = match UserQuery::get_user_by_id(conn, user_id_from_token as i32).await? {
            Some(user) => user,
            None => return Err(Status::invalid_argument("failed_find_user")),
        };

        // Get User projects info
        let user_companies: Vec<UserProject> =
            ProjectQuery::get_user_projects_with_roles(conn, user.id).await?;

        let user_projects_response: Vec<UserProjectsResponse> = user_companies
            .into_iter()
            .map(|c| UserProjectsResponse {
                project_id: c.id,
                project_name: c.name,
                user_role: Some(UserProjectRoleResponse {
                    role_id: c.user_role.id,
                    name: c.user_role.name,
                    description: c.user_role.description.unwrap_or(String::new()),
                }),
            })
            .collect();

        // Create respnse
        let reply = ClearUserResponse {
            user_id: user.id,
            email: user.email,
            user_projects: user_projects_response,
        };

        let response = Response::new(reply);

        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);
        Ok(response)
    }

    async fn register_user(
        &self,
        request: Request<RegisterUserRequest>,
    ) -> Result<Response<NewUserResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

        let conn = &self.connection;

        let request: RegisterUserRequest = request.into_inner();

        // Check if there correct data in request
        let login: String = validators::login_format_validation(request.login)?;
        let password: String = validators::password_format_validation(request.password)?;
        let email: Option<String> = if let Some(email) = request.email {
            Some(validators::email_format_validation(email)?)
        } else {
            None
        };

        // Secure password
        let hash_password = hash_password(password.as_str())?;

        // Create new user
        let new_user = UserQuery::create_new_user(conn, login, hash_password.0, email).await?;

        event!(target: "hellai_app_core_events", Level::DEBUG, "New user:\n{:?}", new_user);

        // Get User session tokens
        let session_claims: SessionClaims = SessionClaims::new(new_user.id as i64);
        let session_token: String = session_claims.into_token()?;

        let refresh_claims: RefreshClaims = RefreshClaims::new(new_user.id as i64);
        let refresh_token = refresh_claims.into_token()?;

        let reply = NewUserResponse {
            user_id: new_user.id,
            email: new_user.email,
            session_token: session_token,
            refresh_token: refresh_token,
        };

        let response = Response::new(reply);

        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);

        Ok(response)
    }

    async fn refresh_session_token(
        &self,
        request: Request<RefreshSessionTokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

        let request: RefreshSessionTokenRequest = request.into_inner();

        let refresh_token: String = request.refresh_token;

        let request_claims = RefreshClaims::from_token(refresh_token)?;

        let session_claims: SessionClaims = SessionClaims::new(request_claims.sub);
        let session_token = session_claims.into_token()?;

        let reply = TokenResponse {
            session_token: session_token,
        };
        let response = Response::new(reply);

        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", response);

        Ok(response)
    }
}
