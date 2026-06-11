use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use mongodb::{bson::{doc, oid::ObjectId}, Database};
use futures_util::TryStreamExt;
use serde_json::{json, Value};
use crate::models::sla_schedule::SlaSchedule;
use serde::Deserialize;

pub fn router(db: Database) -> Router {
    Router::new()
        .route("/", get(list_sla_schedules).post(create_sla_schedule))
        .with_state(db)
}

#[derive(Debug, Deserialize)]
pub struct CreateSlaScheduleInput {
    pub name: String,
    pub timezone: String,
    pub working_days: Vec<String>,
    pub start_time: String,
    pub end_time: String,
}

async fn list_sla_schedules(
    State(db): State<Database>,
) -> Result<Json<Vec<SlaSchedule>>, (StatusCode, Json<Value>)> {
    let collection = db.collection::<SlaSchedule>("sla_schedules");
    let mut cursor = collection.find(doc! {}).await.map_err(internal_error)?;
    let mut schedules = Vec::new();
    while let Some(schedule) = cursor.try_next().await.map_err(internal_error)? {
        schedules.push(schedule);
    }
    Ok(Json(schedules))
}

async fn create_sla_schedule(
    State(db): State<Database>,
    Json(input): Json<CreateSlaScheduleInput>,
) -> Result<(StatusCode, Json<SlaSchedule>), (StatusCode, Json<Value>)> {
    let collection = db.collection::<SlaSchedule>("sla_schedules");
    let new_schedule = SlaSchedule {
        id: ObjectId::new(),
        name: input.name,
        timezone: input.timezone,
        working_days: input.working_days,
        start_time: input.start_time,
        end_time: input.end_time,
    };
    collection.insert_one(&new_schedule).await.map_err(internal_error)?;
    Ok((StatusCode::CREATED, Json(new_schedule)))
}

fn internal_error(e: mongodb::error::Error) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": e.to_string() })),
    )
}
