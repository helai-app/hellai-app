use core_database::queries::{
    projects_query::{ProjectQuery, UserProject},
    user_query::UserQuery,
};
use core_debugger::tracing::{event, Level};
use helai_api_core_service::{
    user_service_server::UserService, AuthenticateWithPasswordRequest, RefreshSessionTokenRequest,
    RegisterUserRequest, TokenResponse, UserResponse,
};

use middleware::auth_token::{RefreshClaims, SessionClaims};

use service::password_validation::{hash_password, verify_hash_password};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service::{
        self, NewUserResponse, UserProjectRoleResponse, UserProjectsResponse,
    },
    middleware::{self, validators},
    MyServer,
};

#[tonic::async_trait]
impl UserService for MyServer {
    async fn authenticate_with_password(
        &self,
        request: Request<AuthenticateWithPasswordRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        event!(target: "hellai_app_core_events", Level::DEBUG, "{:?}", request);

        let conn = &self.connection;

        let request = request.into_inner();

        // Check if there correct data in request
        let login: String = validators::login_format_validation(request.login)?;
        let password: String = validators::password_format_validation(request.password)?;

        // Get user info from bd
        let user_with_password = match UserQuery::get_user_by_login(conn, login).await? {
            Some(user) => user,
            None => return Err(Status::invalid_argument("failed_find_user")),
        };

        let user = user_with_password.0;

        // Check that password from request is same as user set
        if !verify_hash_password(&user_with_password.1.password_hash, password.as_str())? {
            return Err(Status::invalid_argument("credential_failed"));
        }

        // Get User session tokens
        let session_claims: SessionClaims = SessionClaims::new(user.id as i64);
        let session_token: String = session_claims.into_token()?;

        let refresh_claims: RefreshClaims = RefreshClaims::new(user.id as i64);
        let refresh_token = refresh_claims.into_token()?;

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
        let reply = UserResponse {
            user_id: user.id,
            email: user.email,
            session_token: session_token,
            refresh_token: refresh_token,
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
