use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use mongodb::{bson::{doc, oid::ObjectId}, Database};
use futures_util::TryStreamExt;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use crate::models::sla_cache::SlaCache;
use serde::Deserialize;

pub fn router(db: Database) -> Router {
    Router::new()
        .route("/", get(list_sla_caches).post(create_sla_cache))
        .route("/:id", get(get_sla_cache))
        .with_state(db)
}

#[derive(Debug, Deserialize)]
pub struct CreateSlaCacheInput {
    pub issue_id: i32,
    pub project_id: i32,
    pub sla_level_id: String,
    pub schedule_id: String,
    pub response_deadline: Option<DateTime<Utc>>,
    pub resolution_deadline: Option<DateTime<Utc>>,
    pub response_met: bool,
    pub resolution_met: bool,
    pub status: String,
}

async fn list_sla_caches(
    State(db): State<Database>,
) -> Result<Json<Vec<SlaCache>>, (StatusCode, Json<Value>)> {
    let collection = db.collection::<SlaCache>("sla_caches");
    let mut cursor = collection.find(doc! {}).await.map_err(internal_error)?;
    let mut caches = Vec::new();
    while let Some(cache) = cursor.try_next().await.map_err(internal_error)? {
        caches.push(cache);
    }
    Ok(Json(caches))
}

async fn create_sla_cache(
    State(db): State<Database>,
    Json(input): Json<CreateSlaCacheInput>,
) -> Result<(StatusCode, Json<SlaCache>), (StatusCode, Json<Value>)> {
    let collection = db.collection::<SlaCache>("sla_caches");
    
    let sla_level_id = ObjectId::parse_str(&input.sla_level_id)
        .map_err(|_| bad_request("Invalid sla_level_id ObjectId format"))?;
    let schedule_id = ObjectId::parse_str(&input.schedule_id)
        .map_err(|_| bad_request("Invalid schedule_id ObjectId format"))?;

    let new_cache = SlaCache {
        id: ObjectId::new(),
        issue_id: input.issue_id,
        project_id: input.project_id,
        sla_level_id,
        schedule_id,
        response_deadline: input.response_deadline,
        resolution_deadline: input.resolution_deadline,
        response_met: input.response_met,
        resolution_met: input.resolution_met,
        status: input.status,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    collection.insert_one(&new_cache).await.map_err(internal_error)?;
    Ok((StatusCode::CREATED, Json(new_cache)))
}

async fn get_sla_cache(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<Json<SlaCache>, (StatusCode, Json<Value>)> {
    let collection = db.collection::<SlaCache>("sla_caches");
    let object_id = ObjectId::parse_str(&id).map_err(|_| bad_request("Invalid ObjectId format"))?;
    let filter = doc! { "_id": object_id };
    let cache = collection.find_one(filter).await.map_err(internal_error)?;
    match cache {
        Some(c) => Ok(Json(c)),
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
