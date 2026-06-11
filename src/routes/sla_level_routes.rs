use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use mongodb::{bson::{doc, oid::ObjectId}, Database};
use futures_util::TryStreamExt;
use serde_json::{json, Value};
use crate::models::sla_level::SlaLevel;
use serde::Deserialize;

pub fn router(db: Database) -> Router {
    Router::new()
        .route("/", get(list_sla_levels).post(create_sla_level))
        .route("/:id", get(get_sla_level))
        .with_state(db)
}

#[derive(Debug, Deserialize)]
pub struct CreateSlaLevelInput {
    pub sla_id: String,
    pub name: String,
    pub response_time: i32,
    pub resolution_time: i32,
    pub priority: i32,
}

async fn list_sla_levels(
    State(db): State<Database>,
) -> Result<Json<Vec<SlaLevel>>, (StatusCode, Json<Value>)> {
    let collection = db.collection::<SlaLevel>("sla_levels");
    let mut cursor = collection.find(doc! {}).await.map_err(internal_error)?;
    let mut levels = Vec::new();
    while let Some(level) = cursor.try_next().await.map_err(internal_error)? {
        levels.push(level);
    }
    Ok(Json(levels))
}

async fn create_sla_level(
    State(db): State<Database>,
    Json(input): Json<CreateSlaLevelInput>,
) -> Result<(StatusCode, Json<SlaLevel>), (StatusCode, Json<Value>)> {
    let collection = db.collection::<SlaLevel>("sla_levels");
    let sla_id = ObjectId::parse_str(&input.sla_id)
        .map_err(|_| bad_request("Invalid sla_id ObjectId format"))?;
        
    let new_level = SlaLevel {
        id: ObjectId::new(),
        sla_id,
        name: input.name,
        response_time: input.response_time,
        resolution_time: input.resolution_time,
        priority: input.priority,
    };
    collection.insert_one(&new_level).await.map_err(internal_error)?;
    Ok((StatusCode::CREATED, Json(new_level)))
}

async fn get_sla_level(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<Json<SlaLevel>, (StatusCode, Json<Value>)> {
    let collection = db.collection::<SlaLevel>("sla_levels");
    let object_id = ObjectId::parse_str(&id).map_err(|_| bad_request("Invalid ObjectId format"))?;
    let filter = doc! { "_id": object_id };
    let level = collection.find_one(filter).await.map_err(internal_error)?;
    match level {
        Some(l) => Ok(Json(l)),
        None => Err(not_found()),
    }
}

fn internal_error(e: mongodb::error::Error) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": e.to_string() })),
    )
}

fn not_found() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "Document not found" })),
    )
}

fn bad_request(msg: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": msg })),
    )
}
