use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use mongodb::Database;
use shared::SlaError;
use models::{Calendar, SlaDefinition, SlaCache};
use transition_reconstructor::{Transition, reconstruct_intervals};
use calculation_engine::calculate_sla;
use cache_manager::save_cache;
use audit_logger::log_event;
use chrono::Utc;

#[derive(Clone)]
pub struct RecalculateTask {
    pub issue_id: i32,
    pub project_id: i32,
    pub tracker_id: i32,
    pub priority: String,
    pub initial_status_id: i32,
    pub initial_status_name: String,
    pub transitions: Vec<Transition>,
}

#[derive(Clone)]
pub struct EventDispatcher {
    tx: UnboundedSender<RecalculateTask>,
}

impl EventDispatcher {
    pub fn new() -> (Self, UnboundedReceiver<RecalculateTask>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (EventDispatcher { tx }, rx)
    }

    pub fn dispatch(&self, task: RecalculateTask) -> Result<(), SlaError> {
        self.tx
            .send(task)
            .map_err(|e| SlaError::InternalError(e.to_string()))
    }
}

pub async fn run_worker(db: Database, mut rx: UnboundedReceiver<RecalculateTask>) {
    println!("Background Event Dispatcher worker started...");
    while let Some(task) = rx.recv().await {
        println!("Received recalculation task for issue_id: {}", task.issue_id);
        if let Err(e) = process_recalculate(&db, task).await {
            eprintln!("Failed to process recalculate task: {:?}", e);
        }
    }
}

async fn process_recalculate(db: &Database, task: RecalculateTask) -> Result<(), SlaError> {
    // 1. Find the SLA Definition matching the project and tracker
    let sla_definitions_col = db.collection::<SlaDefinition>("sla_definitions");
    
    let filter = mongodb::bson::doc! {
        "project_tracker_mappings": {
            "$elemMatch": {
                "project_id": task.project_id,
                "tracker_id": task.tracker_id
            }
        }
    };
    
    let sla_def = sla_definitions_col
        .find_one(filter)
        .await
        .map_err(|e| SlaError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SlaError::NotFound(format!("No SLA Definition found for project_id {} and tracker_id {}", task.project_id, task.tracker_id)))?;

    // 2. Fetch the associated Calendar
    let calendars_col = db.collection::<Calendar>("calendars");
    let calendar = calendars_col
        .find_one(mongodb::bson::doc! { "_id": sla_def.calendar_id })
        .await
        .map_err(|e| SlaError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SlaError::NotFound(format!("Calendar with ID {} not found", sla_def.calendar_id)))?;

    // 3. Reconstruct Status Intervals
    let created_at = task.transitions.first().map(|t| t.transitioned_at).unwrap_or_else(Utc::now);
    let intervals = reconstruct_intervals(
        created_at,
        task.initial_status_id,
        task.initial_status_name,
        task.transitions,
        Utc::now(),
    );

    // 4. Perform SLA target/spent calculations
    let calc_result = calculate_sla(&sla_def, &calendar, &intervals, &task.priority, Utc::now())
        .ok_or_else(|| SlaError::InternalError("Failed to calculate SLA targets".to_string()))?;

    // 5. Build and Save SlaCache
    let sla_cache = SlaCache {
        id: mongodb::bson::oid::ObjectId::new(),
        issue_id: task.issue_id,
        project_id: task.project_id,
        tracker_id: task.tracker_id,
        sla_definition_id: sla_def.id,
        level_name: calc_result.level_name,
        response_deadline: calc_result.response_deadline,
        resolution_deadline: calc_result.resolution_deadline,
        response_met: calc_result.response_met,
        resolution_met: calc_result.resolution_met,
        status: calc_result.status.clone(),
        spent_minutes_by_status: calc_result.spent_minutes_by_status,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    save_cache(db, &sla_cache).await?;

    // 6. Log event to audit stream
    let audit_desc = format!(
        "Recalculated SLA for issue {}. Status: {}. Response Met: {}. Resolution Met: {}.",
        task.issue_id,
        calc_result.status,
        calc_result.response_met,
        calc_result.resolution_met
    );
    log_event(db, task.issue_id, "Recalculated", &audit_desc).await?;

    println!("Successfully recalculated SLA for issue_id: {}", task.issue_id);
    Ok(())
}
