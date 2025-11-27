/// Unified model type definition shared across the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model3D {
    DreamScene360,
    GaussianDreamerPro,
    TripoSR,
    SceneScape,
}

impl Model3D {
    /// Model name for display in UI
    pub fn name(&self) -> &str {
        match self {
            Self::DreamScene360 => "DreamScene360",
            Self::GaussianDreamerPro => "GaussianDreamerPro",
            Self::TripoSR => "TripoSR",
            Self::SceneScape => "SceneScape",
        }
    }

    /// Model ID for API communication
    pub fn id(&self) -> &str {
        match self {
            Self::DreamScene360 => "dreamscene360",
            Self::GaussianDreamerPro => "gaussiandreamerpro",
            Self::TripoSR => "triposr",
            Self::SceneScape => "scenescape",
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &str {
        match self {
            Self::DreamScene360 => "360° explorable scenes (2-5 min)",
            Self::GaussianDreamerPro => "High-quality objects (10-15 min)",
            Self::TripoSR => "Fast preview (<1 sec)",
            Self::SceneScape => "Complex multi-object scenes (3-7 min)",
        }
    }

    /// UI icon
    pub fn icon(&self) -> &str {
        match self {
            Self::DreamScene360 => "🌐",
            Self::GaussianDreamerPro => "💎",
            Self::TripoSR => "⚡",
            Self::SceneScape => "🏗️",
        }
    }

    /// Model type (scene vs object)
    pub fn model_type(&self) -> ModelType {
        match self {
            Self::DreamScene360 | Self::SceneScape => ModelType::Scene,
            Self::GaussianDreamerPro | Self::TripoSR => ModelType::Object,
        }
    }

    /// Estimated generation time in seconds
    pub fn estimated_time_secs(&self) -> u32 {
        match self {
            Self::DreamScene360 => 180,      // 3 min
            Self::GaussianDreamerPro => 750, // 12.5 min
            Self::TripoSR => 1,              // <1 sec
            Self::SceneScape => 300,         // 5 min
        }
    }

    /// Quality tier
    pub fn quality(&self) -> Quality {
        match self {
            Self::DreamScene360 => Quality::High,
            Self::GaussianDreamerPro => Quality::VeryHigh,
            Self::TripoSR => Quality::Medium,
            Self::SceneScape => Quality::High,
        }
    }

    /// All available models
    pub fn all() -> [Model3D; 4] {
        [
            Self::DreamScene360,
            Self::GaussianDreamerPro,
            Self::TripoSR,
            Self::SceneScape,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    Scene,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Quality {
    Medium,
    High,
    VeryHigh,
}

impl Default for Model3D {
    fn default() -> Self {
        Self::DreamScene360
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_ids() {
        assert_eq!(Model3D::DreamScene360.id(), "dreamscene360");
        assert_eq!(Model3D::GaussianDreamerPro.id(), "gaussiandreamerpro");
    }

    #[test]
    fn test_model_types() {
        assert_eq!(Model3D::DreamScene360.model_type(), ModelType::Scene);
        assert_eq!(Model3D::TripoSR.model_type(), ModelType::Object);
    }

    #[test]
    fn test_all_models() {
        assert_eq!(Model3D::all().len(), 4);
    }
}