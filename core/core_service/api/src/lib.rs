use std::env;

use helai_api_core_service::{
    user_service_server::{UserService, UserServiceServer},
    AuthenticateWithPasswordRequest, RefreshSessionTokenRequest, RegisterUserRequest,
    TokenResponse, UserResponse, UserRole,
};
use middleware::auth_token::{RefreshClaims, SessionClaims};
use tonic::{transport::Server, Request, Response, Status};

use sea_orm::{Database, DatabaseConnection};

mod middleware;

/// For init proto generation
pub mod helai_api_core_service {
    tonic::include_proto!("helai_api_core_service");
}

#[derive(Default)]
pub struct MyServer {
    connection: DatabaseConnection,
}

#[tonic::async_trait]
impl UserService for MyServer {
    async fn authenticate_with_password(
        &self,
        request: Request<AuthenticateWithPasswordRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let conn = &self.connection;

        let user_id: i64 = 1;
        let session_id: i64 = 1;

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
            user_role: UserRole::User.into(),
            email: "test@test.com".into(),
            session_token: session_token,
            refresh_token: refresh_token,
        };

        Ok(Response::new(reply))
    }

    async fn register_user(
        &self,
        request: Request<RegisterUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = UserResponse {
            user_id: 1,
            user_role: UserRole::User.into(),
            email: "test@test.com".into(),
            session_token: "".into(),
            refresh_token: "".into(),
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

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50052".parse().unwrap();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // establish database connection
    let connection = Database::connect(&database_url).await?;

    let my_server = MyServer { connection };

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(my_server))
        .serve(addr)
        .await?;

    Ok(())
}
