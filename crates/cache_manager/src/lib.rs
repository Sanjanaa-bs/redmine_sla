use mongodb::{Database, bson::doc};
use models::SlaCache;
use shared::SlaError;

pub async fn get_cache_by_issue_id(
    db: &Database,
    issue_id: i32,
) -> Result<Option<SlaCache>, SlaError> {
    let collection = db.collection::<SlaCache>("sla_caches");
    collection
        .find_one(doc! { "issue_id": issue_id })
        .await
        .map_err(|e| SlaError::DatabaseError(e.to_string()))
}

pub async fn save_cache(
    db: &Database,
    cache: &SlaCache,
) -> Result<(), SlaError> {
    let collection = db.collection::<SlaCache>("sla_caches");
    let query = doc! { "issue_id": cache.issue_id };
    
    let existing = collection.find_one(query.clone()).await
        .map_err(|e| SlaError::DatabaseError(e.to_string()))?;

    if existing.is_some() {
        collection
            .replace_one(query, cache)
            .await
            .map_err(|e| SlaError::DatabaseError(e.to_string()))?;
    } else {
        collection
            .insert_one(cache)
            .await
            .map_err(|e| SlaError::DatabaseError(e.to_string()))?;
    }
    
    Ok(())
}
