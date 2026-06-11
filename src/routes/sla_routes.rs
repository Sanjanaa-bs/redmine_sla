use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use mongodb::{bson::{doc, oid::ObjectId}, Database};
use futures_util::TryStreamExt;
use chrono::Utc;
use serde_json::{json, Value};
use crate::models::sla::Sla;
use serde::Deserialize;

pub fn router(db: Database) -> Router {
    Router::new()
        .route("/", get(list_slas).post(create_sla))
        .route("/:id", get(get_sla).delete(delete_sla))
        .with_state(db)
}

#[derive(Debug, Deserialize)]
pub struct CreateSlaInput {
    pub name: String,
    pub description: Option<String>,
}

async fn list_slas(
    State(db): State<Database>,
) -> Result<Json<Vec<Sla>>, (StatusCode, Json<Value>)> {
    let collection = db.collection::<Sla>("slas");
    let mut cursor = collection.find(doc! {}).await.map_err(internal_error)?;
    let mut slas = Vec::new();
    while let Some(sla) = cursor.try_next().await.map_err(internal_error)? {
        slas.push(sla);
    }
    Ok(Json(slas))
}

async fn create_sla(
    State(db): State<Database>,
    Json(input): Json<CreateSlaInput>,
) -> Result<(StatusCode, Json<Sla>), (StatusCode, Json<Value>)> {
    let collection = db.collection::<Sla>("slas");
    let new_sla = Sla {
        id: ObjectId::new(),
        name: input.name,
        description: input.description,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    collection.insert_one(&new_sla).await.map_err(internal_error)?;
    Ok((StatusCode::CREATED, Json(new_sla)))
}

async fn get_sla(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<Json<Sla>, (StatusCode, Json<Value>)> {
    let collection = db.collection::<Sla>("slas");
    let object_id = ObjectId::parse_str(&id).map_err(|_| bad_request("Invalid ObjectId format"))?;
    let filter = doc! { "_id": object_id };
    let sla = collection.find_one(filter).await.map_err(internal_error)?;
    match sla {
        Some(s) => Ok(Json(s)),
        None => Err(not_found()),
    }
}

async fn delete_sla(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let collection = db.collection::<Sla>("slas");
    let object_id = ObjectId::parse_str(&id).map_err(|_| bad_request("Invalid ObjectId format"))?;
    let filter = doc! { "_id": object_id };
    let result = collection.delete_one(filter).await.map_err(internal_error)?;
    if result.deleted_count == 1 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found())
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
