use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaCacheSpent {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub cache_id: ObjectId,
    pub issue_id: i32,
    pub status_id: i32,
    pub status_name: String,
    pub time_spent: i32,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}
