use std::error::Error;

use colored::Colorize;
use core_debugger::init_tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    println!("{}", "\n===============================".blue().bold());
    println!("🔧 Loading environment variables...");

    dotenvy::dotenv()?;

    println!("✅ Environment variables loaded successfully.");

    // Start the API server.
    println!("{}", "\n===============================".blue().bold());
    println!("🚀 Starting core service...\n");

    // Initialize tracing for the application.
    init_tracing();

    let result = api::start().await;

    // Check if the API server starts without errors.
    if let Err(err) = result {
        println!(
            "{}",
            format!("❌ Error while running the server: {}", err)
                .red()
                .bold()
        );
    } else {
        println!("✅ Server shutdown successfully.\n");
    }

    println!("{}", "===============================\n".blue().bold());

    Ok(())
}
