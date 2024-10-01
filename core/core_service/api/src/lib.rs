use std::env;

use colored::Colorize;
use helai_api_core_service::user_service_server::UserServiceServer;

use migration::{Migrator, MigratorTrait};
use tonic::transport::Server;

use sea_orm::Database;

use my_server::MyServer;
// use services::user_service::UserService;

mod middleware;
mod my_server;
mod services;

/// For init proto generation
pub mod helai_api_core_service {
    tonic::include_proto!("helai_api_core_service");
}

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50052".parse().unwrap();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("{}", "\n===============================".blue().bold());
    println!("🚀 Starting server...");

    // establish database connection
    let connection = Database::connect(&database_url).await?;
    println!("✅ Database connected successfully");

    Migrator::up(&connection, None).await?;
    println!("📦 Database migrations applied");

    let my_server = MyServer { connection };

    println!("{}", "\n===============================".blue().bold());
    println!(
        "✨ {} {}",
        "GreeterServer".green().bold(),
        format!("listening on {}", addr).yellow()
    );
    println!("{}", "===============================\n".blue().bold());

    Server::builder()
        .add_service(UserServiceServer::new(my_server))
        .serve(addr)
        .await?;

    Ok(())
}
