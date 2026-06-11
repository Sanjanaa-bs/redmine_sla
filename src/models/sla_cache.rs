use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaCache {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub issue_id: i32,
    pub project_id: i32,
    pub sla_level_id: ObjectId,
    pub schedule_id: ObjectId,
    pub response_deadline: Option<DateTime<Utc>>,
    pub resolution_deadline: Option<DateTime<Utc>>,
    pub response_met: bool,
    pub resolution_met: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
