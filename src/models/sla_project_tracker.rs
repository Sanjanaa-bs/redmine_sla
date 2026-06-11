use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaProjectTracker {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub project_id: i32,
    pub tracker_id: i32,
    pub sla_id: ObjectId,
    pub schedule_id: ObjectId,
}
