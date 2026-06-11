use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaHoliday {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub date: DateTime<Utc>,
}
