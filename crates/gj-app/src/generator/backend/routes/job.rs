use std::sync::Arc;
use axum::extract::{Request, State};
use axum::Json;
use axum::response::IntoResponse;
use crate::generator::backend::schemas::JobStatusResponse;
use crate::generator::backend::state::GenState;

pub async fn poll_job_status(
    State(state): State<Arc<GenState>>,
    Json(resp): Json<JobStatusResponse>,
) -> impl IntoResponse {
    state.emit_job_status(resp)?;
}
