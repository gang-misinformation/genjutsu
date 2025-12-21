use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Queued,
    Submitting,
    Generating,
    Complete,
    Failed,
}

impl JobStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Submitting | Self::Generating)
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete | Self::Failed)
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Queued => "⏳",
            Self::Submitting => "📤",
            Self::Generating => "⚡",
            Self::Complete => "✅",
            Self::Failed => "❌",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Queued => egui::Color32::GRAY,
            Self::Submitting => egui::Color32::LIGHT_BLUE,
            Self::Generating => egui::Color32::YELLOW,
            Self::Complete => egui::Color32::GREEN,
            Self::Failed => egui::Color32::RED,
        }
    }
}