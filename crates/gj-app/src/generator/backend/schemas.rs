use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationJob {
    pub prompt: String,
    pub model: String,
    pub guidance_scale: f32,
    pub num_inference_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Pending,
    Started,
    Success,
    Failure,
    Retry,
    Revoked,
}

impl JobStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Pending | Self::Started)
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Success | Self::Failure)
    }
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