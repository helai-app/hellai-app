use colored::Colorize;
use helai_api_core_service::companies_service_server::CompaniesServiceServer;
use helai_api_core_service::notes_service_server::NotesServiceServer;
use helai_api_core_service::projects_service_server::ProjectsServiceServer;
use helai_api_core_service::tasks_service_server::TasksServiceServer;
use helai_api_core_service::user_service_server::UserServiceServer;
use http::Method;
use std::{env, sync::Arc};

use migration::{Migrator, MigratorTrait};
use tonic::transport::Server;

use sea_orm::Database;

use my_server::MyServer;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
// use tonic_web::GrpcWebLayer;
// use services::user_service::UserService;

mod middleware;
mod my_server;
mod services;

/// For init proto generation
pub mod helai_api_core_service {
    tonic::include_proto!("helai_api_core_service");
}

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50053".parse().unwrap();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("{}", "\n===============================".blue().bold());
    println!("🚀 Starting server...");

    // establish database connection
    let connection = Database::connect(&database_url).await?;
    println!("✅ Database connected successfully");

    Migrator::up(&connection, None).await?;
    println!("📦 Database migrations applied");

    let my_server = MyServer {
        connection: Arc::new(connection),
        // other fields
    };

    println!("{}", "\n===============================".blue().bold());
    println!(
        "✨ {} {}",
        "GreeterServer".green().bold(),
        format!("listening on {}", addr).yellow()
    );
    println!("{}", "===============================\n".blue().bold());

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
        // allow specific headers, including Content-Type
        .allow_headers(Any);

    Server::builder()
        .accept_http1(true)
        // .layer(GrpcWebLayer::new())
        .layer(cors)
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(UserServiceServer::new(my_server.clone()))
        .add_service(ProjectsServiceServer::new(my_server.clone()))
        .add_service(CompaniesServiceServer::new(my_server.clone()))
        .add_service(TasksServiceServer::new(my_server.clone()))
        .add_service(NotesServiceServer::new(my_server.clone()))
        .serve(addr)
        .await?;

    Ok(())
}
