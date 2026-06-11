use mongodb::Database;
use models::SlaEventLog;
use shared::SlaError;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub async fn log_event(
    db: &Database,
    issue_id: i32,
    event_type: &str,
    description: &str,
) -> Result<(), SlaError> {
    let collection = db.collection::<SlaEventLog>("sla_event_logs");
    
    let event = SlaEventLog {
        id: ObjectId::new(),
        issue_id,
        event_type: event_type.to_string(),
        description: description.to_string(),
        timestamp: Utc::now(),
    };
    
    collection
        .insert_one(&event)
        .await
        .map_err(|e| SlaError::DatabaseError(e.to_string()))?;
        
    Ok(())
}
