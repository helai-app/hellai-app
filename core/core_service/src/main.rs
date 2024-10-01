use std::error::Error;

use core_debugger::init_tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    dotenvy::dotenv()?;

    init_tracing();

    let result = api::start().await;
    if let Some(err) = result.err() {
        println!("Error: {err}");
    }

    Ok(())
}
