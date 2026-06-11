use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaLevel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub sla_id: ObjectId,
    pub name: String,
    pub response_time: i32,
    pub resolution_time: i32,
    pub priority: i32,
}
