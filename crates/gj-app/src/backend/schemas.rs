use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationJob {
    pub prompt: String,
    pub model: String,
    pub guidance_scale: f32,
    pub num_inference_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationResult {
    pub ply_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobResponse {
    pub id: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobStatusResponse {
    pub id: String,
    pub status: String,
    pub progress: Option<f32>,
    pub message: Option<String>,
    pub result: Option<GenerationResult>,
    pub error: Option<String>,
}

pub type JobStatusPair = (String, JobStatusResponse);