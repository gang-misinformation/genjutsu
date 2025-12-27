use serde_json::Value;
use std::fmt;
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;
use crate::generator::db::job::SurrealDatetime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Queued,
    Generating,
    Complete,
    Failed,
}

impl JobStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Generating)
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete | Self::Failed)
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Queued => "⏳",
            Self::Generating => "⚡",
            Self::Complete => "✅",
            Self::Failed => "❌",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Queued => egui::Color32::GRAY,
            Self::Generating => egui::Color32::YELLOW,
            Self::Complete => egui::Color32::GREEN,
            Self::Failed => egui::Color32::RED,
        }
    }
}

impl fmt::Display for JobStatus{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JobStatus::Complete => write!(f, "Complete"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Generating => write!(f, "Generating"),
            JobStatus::Queued => write!(f, "Queued"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, PartialEq)]
pub struct JobInputs {
    pub prompt: String,
    pub model: String,
    pub guidance_scale: f32,
    pub num_inference_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
pub struct JobOutputs {
    pub ply_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
pub struct JobMetadata {
    pub status: JobStatus,
    pub progress: f32,
    pub message: Option<String>,
    pub error: Option<String>,
    pub created_at: SurrealDatetime,
    pub updated_at: SurrealDatetime,
    pub completed_at: Option<SurrealDatetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
pub struct Job {
    pub inputs: JobInputs,
    pub metadata: JobMetadata,
    pub outputs: Option<JobOutputs>
}