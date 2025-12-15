use std::sync::Arc;
use axum::Router;
use axum::routing::{get, put};
use crate::generator::backend::routes::job::poll_job_status;
use crate::generator::backend::state::GenState;

mod job;

pub fn api_routes() -> Router<Arc<GenState>> {
    Router::new()
        .route("/status/:id", put(poll_job_status))
}