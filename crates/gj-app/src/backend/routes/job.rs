use std::sync::Arc;
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::Deserialize;
use crate::backend::schemas::{JobStatusResponse};
use crate::backend::state::GenState;
use crate::job::JobStatus;
use crate::state::AppState;

pub async fn update_job_progress(
    State(state): State<Arc<GenState>>,
    Path(id): Path<String>,
    Json(resp): Json<JobStatusResponse>,
) -> impl IntoResponse {
    state.emit_job_status(id, resp);
}