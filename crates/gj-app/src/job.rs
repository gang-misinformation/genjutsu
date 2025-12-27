use serde_json::Value;
use std::fmt;
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;
use crate::generator::db::job::SurrealDatetime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
pub enum JobStatus {
    QUEUED,
    GENERATING,
    COMPLETE,
    FAILED,
}

impl JobStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::QUEUED | Self::GENERATING)
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Self::COMPLETE | Self::FAILED)
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::QUEUED => "⏳",
            Self::GENERATING => "⚡",
            Self::COMPLETE => "✅",
            Self::FAILED => "❌",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::QUEUED => egui::Color32::GRAY,
            Self::GENERATING => egui::Color32::YELLOW,
            Self::COMPLETE => egui::Color32::GREEN,
            Self::FAILED => egui::Color32::RED,
        }
    }
}

impl fmt::Display for JobStatus{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JobStatus::COMPLETE => write!(f, "Complete"),
            JobStatus::FAILED => write!(f, "Failed"),
            JobStatus::GENERATING => write!(f, "Generating"),
            JobStatus::QUEUED => write!(f, "Queued"),
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