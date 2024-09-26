use helai_api_core_service::{
    user_service_server::UserService, AuthenticateWithPasswordRequest, GlobalUserRole,
    RefreshSessionTokenRequest, RegisterUserRequest, TokenResponse, UserCompanyResponse,
    UserCompanyRoleResponse, UserResponse,
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

        let request = request.into_inner();

        let password = validators::login_format_validation(request.login)?;
        let login = validators::password_format_validation(request.password)?;

        let user_id: i64 = 1;
        let session_id: i64 = 1;

        verify_hash_password(password.as_str(), "admin")?;

        let session_claims: SessionClaims = SessionClaims::new(user_id);
        let session_token = session_claims
            .into_token()
            .expect("Failed to create session JWT token");

        let refresh_claims: RefreshClaims = RefreshClaims::new(session_id, user_id);
        let refresh_token = refresh_claims
            .into_token()
            .expect("Failed to create refresh JWT token");

        let reply = UserResponse {
            user_id: user_id as i32,
            user_role: GlobalUserRole::User.into(),
            email: Some("test@test.com".into()),
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
            user_role: GlobalUserRole::User.into(),
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
