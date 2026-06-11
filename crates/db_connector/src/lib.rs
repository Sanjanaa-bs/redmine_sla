use mongodb::{Client, Database};
use std::env;
use shared::SlaError;

pub async fn connect() -> Result<Database, SlaError> {
    dotenv::dotenv().ok();
    let uri = env::var("MONGODB_URI")
        .map_err(|_| SlaError::ValidationError("MONGODB_URI must be set".to_string()))?;
    let db_name = env::var("DATABASE_NAME")
        .map_err(|_| SlaError::ValidationError("DATABASE_NAME must be set".to_string()))?;

    let client = Client::with_uri_str(&uri)
        .await
        .map_err(|e| SlaError::DatabaseError(e.to_string()))?;

    println!("Connected to MongoDB Database: {}", db_name);
    Ok(client.database(&db_name))
}
