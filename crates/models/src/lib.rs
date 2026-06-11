use serde::{Serialize, Deserialize};
pub use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};

// 1. Calendars Collection Model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Calendar {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub timezone: String,
    pub working_days: Vec<String>, // e.g., ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]
    pub start_time: String,       // e.g., "09:00"
    pub end_time: String,         // e.g., "17:00"
    pub holidays: Vec<Holiday>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Holiday {
    pub name: String,
    pub date: DateTime<Utc>,
}

// 2. SLA Definitions Collection Model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaDefinition {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub description: Option<String>,
    pub sla_type: String, // e.g. "Response" or "Resolution"
    pub levels: Vec<SlaLevel>,
    pub active_statuses: Vec<i32>, // Statuses where target SLA timer is ticking
    pub project_tracker_mappings: Vec<ProjectTrackerMapping>,
    pub calendar_id: ObjectId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaLevel {
    pub name: String,
    pub response_time: i32,    // in minutes
    pub resolution_time: i32,  // in minutes
    pub priority: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectTrackerMapping {
    pub project_id: i32,
    pub tracker_id: i32,
}

// 3. SLA Cache Collection Model (tracks calculations for an issue)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaCache {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub issue_id: i32,
    pub project_id: i32,
    pub tracker_id: i32,
    pub sla_definition_id: ObjectId,
    pub level_name: String, // Matches level priority
    pub response_deadline: Option<DateTime<Utc>>,
    pub resolution_deadline: Option<DateTime<Utc>>,
    pub response_met: bool,
    pub resolution_met: bool,
    pub status: String, // "Active", "Paused", "Completed", "Breached"
    pub spent_minutes_by_status: Vec<SpentStatusTime>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpentStatusTime {
    pub status_id: i32,
    pub status_name: String,
    pub time_spent: i32, // total time in minutes
    pub intervals: Vec<StatusInterval>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusInterval {
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

// 4. SLA Event Logs Collection Model (audit stream)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlaEventLog {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub issue_id: i32,
    pub event_type: String, // e.g. "Recalculated", "Breach", "Met"
    pub description: String,
    pub timestamp: DateTime<Utc>,
}
