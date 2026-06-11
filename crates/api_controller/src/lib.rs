use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use mongodb::Database;
use models::{Calendar, SlaDefinition, SlaCache};
use event_dispatcher::{EventDispatcher, RecalculateTask};
use transition_reconstructor::Transition;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub dispatcher: EventDispatcher,
}

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/calendars", post(create_calendar))
        .route("/api/sla_definitions", post(create_sla_definition))
        .route("/api/issues/:id/recalculate", post(recalculate_issue))
        .route("/api/issues/:id/sla", get(get_issue_sla))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct RecalculateRequest {
    pub project_id: i32,
    pub tracker_id: i32,
    pub priority: String,
    pub initial_status_id: i32,
    pub initial_status_name: String,
    pub transitions: Vec<Transition>,
}

async fn recalculate_issue(
    State(state): State<AppState>,
    Path(issue_id_str): Path<String>,
    Json(input): Json<RecalculateRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let issue_id = issue_id_str
        .parse::<i32>()
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid issue_id format" }))))?;

    let task = RecalculateTask {
        issue_id,
        project_id: input.project_id,
        tracker_id: input.tracker_id,
        priority: input.priority,
        initial_status_id: input.initial_status_id,
        initial_status_name: input.initial_status_name,
        transitions: input.transitions,
    };

    state.dispatcher.dispatch(task).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "message": "Recalculation task queued successfully" })),
    ))
}

async fn get_issue_sla(
    State(state): State<AppState>,
    Path(issue_id_str): Path<String>,
) -> Result<Json<SlaCache>, (StatusCode, Json<Value>)> {
    let issue_id = issue_id_str
        .parse::<i32>()
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid issue_id format" }))))?;

    let cache = cache_manager::get_cache_by_issue_id(&state.db, issue_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    match cache {
        Some(c) => Ok(Json(c)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "SLA Cache not found for this issue" })),
        )),
    }
}

async fn create_calendar(
    State(state): State<AppState>,
    Json(calendar): Json<Calendar>,
) -> Result<(StatusCode, Json<Calendar>), (StatusCode, Json<Value>)> {
    let collection = state.db.collection::<Calendar>("calendars");
    collection
        .insert_one(&calendar)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    Ok((StatusCode::CREATED, Json(calendar)))
}

async fn create_sla_definition(
    State(state): State<AppState>,
    Json(definition): Json<SlaDefinition>,
) -> Result<(StatusCode, Json<SlaDefinition>), (StatusCode, Json<Value>)> {
    let collection = state.db.collection::<SlaDefinition>("sla_definitions");
    collection
        .insert_one(&definition)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    Ok((StatusCode::CREATED, Json(definition)))
}
