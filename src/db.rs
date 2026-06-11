use mongodb::{Client, Database};
use std::env;

pub async fn connect() -> Database {
    dotenv::dotenv().ok();
    let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");

    let client = Client::with_uri_str(&uri)
        .await
        .expect("Failed to connect to MongoDB");

    println!("Connected to MongoDB");
    client.database(&db_name)
}
