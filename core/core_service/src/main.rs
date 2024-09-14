use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    dotenvy::dotenv()?;

    let result = api::start().await;
    if let Some(err) = result.err() {
        println!("Error: {err}");
    }

    Ok(())
}
