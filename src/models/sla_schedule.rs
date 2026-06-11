use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaSchedule {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub timezone: String,
    pub working_days: Vec<String>,
    pub start_time: String,
    pub end_time: String,
}
