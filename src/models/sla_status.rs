use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaStatus {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub sla_id: ObjectId,
    pub status_id: i32,
    pub status_name: String,
    pub is_active: bool,
}
