use dotenv::dotenv;
use tokio::net::TcpListener;
use api_controller::{create_routes, AppState};
use event_dispatcher::{EventDispatcher, run_worker};

#[tokio::main]
async fn main() {
    // Load environment variables from .env
    dotenv().ok();

    // Initialize MongoDB Connection
    let db = db_connector::connect()
        .await
        .expect("Failed to connect to MongoDB");

    // Initialize Event Dispatcher & worker queue
    let (dispatcher, rx) = EventDispatcher::new();

    // Spawn Background Recalculation Worker
    tokio::spawn(run_worker(db.clone(), rx));

    // Create Axum Router
    let app = create_routes(AppState {
        db,
        dispatcher,
    });

    // Read port from env or default to 8080
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let server_url = format!("http://localhost:{}", port);

    println!("Starting SLA Engine Workspace server on {}", server_url);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
