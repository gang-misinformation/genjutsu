use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::job::JobStatus;
use chrono::{DateTime, Utc, Duration};
use surrealdb::sql::Datetime as SurrealDatetime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: Option<String>,
    pub job_id: String,
    pub prompt: String,
    pub model: String,
    pub status: JobStatus,
    pub guidance_scale: f32,
    pub num_inference_steps: u32,
    pub created_at: SurrealDatetime,
    pub updated_at: SurrealDatetime,
    pub completed_at: Option<SurrealDatetime>,
    pub progress: f32,
    pub message: Option<String>,
    pub ply_path: Option<PathBuf>,
    pub error: Option<String>,
}