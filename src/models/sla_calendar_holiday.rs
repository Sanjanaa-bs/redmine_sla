use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaCalendarHoliday {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub schedule_id: ObjectId,
    pub holiday_id: ObjectId,
}
