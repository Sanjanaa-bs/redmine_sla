mod db;
mod models;
mod routes;

use tokio::net::TcpListener;
use dotenv::dotenv;
use routes::create_routes;

#[tokio::main]
async fn main() {
    // Load environment variables from .env
    dotenv().ok();

    // Initialize MongoDB Connection
    let db = db::connect().await;

    // Create Axum Router
    let app = create_routes(db);

    // Read port from env or default to 8080
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let server_url = format!("http://localhost:{}", port);

    println!("Starting SLA server on {}", server_url);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
