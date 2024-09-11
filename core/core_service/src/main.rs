#[tokio::main]
async fn main() {
    let result = api::start().await;
    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
