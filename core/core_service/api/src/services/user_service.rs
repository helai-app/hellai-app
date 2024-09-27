use core_database::queries::user_query::UserQuery;
use helai_api_core_service::{
    user_service_server::UserService, AuthenticateWithPasswordRequest, RefreshSessionTokenRequest,
    RegisterUserRequest, TokenResponse, UserCompanyResponse, UserCompanyRoleResponse, UserResponse,
};

use middleware::auth_token::{RefreshClaims, SessionClaims};

use service::password_validation::{hash_password, verify_hash_password};
use tonic::{Request, Response, Status};

use crate::{
    helai_api_core_service,
    middleware::{self, validators},
    MyServer,
};

#[tonic::async_trait]
impl UserService for MyServer {
    async fn authenticate_with_password(
        &self,
        request: Request<AuthenticateWithPasswordRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        println!("Got a request: {:?}", request);

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
        verify_hash_password(&user_with_password.1.password_hash, password.as_str())?;

        // Get User session tokens
        let session_claims: SessionClaims = SessionClaims::new(user.id as i64);
        let session_token: String = session_claims.into_token()?;

        let refresh_claims: RefreshClaims = RefreshClaims::new(user.id as i64);
        let refresh_token = refresh_claims.into_token()?;

        let reply = UserResponse {
            user_id: user.id,
            email: user.email,
            session_token: session_token,
            refresh_token: refresh_token,
            user_companies: vec![UserCompanyResponse {
                company_id: 1,
                company_name: "test".to_string(),
                user_role: Some(UserCompanyRoleResponse {
                    role_id: 1,
                    name: "test".to_string(),
                    description: "".to_string(),
                }),
            }],
        };

        Ok(Response::new(reply))
    }

    async fn register_user(
        &self,
        request: Request<RegisterUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let request_data = request.into_inner();

        let password_data = hash_password(&request_data.password)?;

        println!("Hash:{}\nSalt:{}", password_data.0, password_data.1);

        let reply = UserResponse {
            user_id: 1,
            email: Some("test@test.com".into()),
            session_token: "".into(),
            refresh_token: "".into(),
            user_companies: vec![UserCompanyResponse {
                company_id: 1,
                company_name: "test".to_string(),
                user_role: Some(UserCompanyRoleResponse {
                    role_id: 1,
                    name: "test".to_string(),
                    description: "".to_string(),
                }),
            }],
        };

        Ok(Response::new(reply))
    }

    async fn refresh_session_token(
        &self,
        request: Request<RefreshSessionTokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let refresh_token: String = request.into_inner().refresh_token;

        let request_claims = RefreshClaims::from_token(refresh_token)?;

        let session_claims: SessionClaims = SessionClaims::new(request_claims.sub);
        let session_token = session_claims.into_token()?;

        let reply = TokenResponse {
            session_token: session_token,
        };

        Ok(Response::new(reply))
    }
}
