pub mod sla_routes;
pub mod sla_level_routes;
pub mod sla_schedule_routes;
pub mod sla_cache_routes;

use axum::Router;
use mongodb::Database;

pub fn create_routes(db: Database) -> Router {
    Router::new()
        .nest("/api/slas", sla_routes::router(db.clone()))
        .nest("/api/sla_levels", sla_level_routes::router(db.clone()))
        .nest("/api/sla_schedules", sla_schedule_routes::router(db.clone()))
        .nest("/api/sla_caches", sla_cache_routes::router(db.clone()))
}
