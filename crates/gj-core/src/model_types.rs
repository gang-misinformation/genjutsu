/// Unified model type definition shared across the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model3D {
    ShapE,
}

impl Model3D {
    /// Model name for display in UI
    pub fn name(&self) -> &str {
        match self {
            Self::ShapE => "Shap-E",
        }
    }

    /// Model ID for API communication
    pub fn id(&self) -> &str {
        match self {
            Self::ShapE => "shap_e",
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &str {
        match self {
            Self::ShapE => "OpenAI's Shap-E - Fast text-to-3D (30-60 sec)",
        }
    }

    /// UI icon
    pub fn icon(&self) -> &str {
        match self {
            Self::ShapE => "⚡",
        }
    }

    /// Model type (scene vs object)
    pub fn model_type(&self) -> ModelType {
        match self {
            Self::ShapE => ModelType::Object,
        }
    }

    /// Estimated generation time in seconds
    pub fn estimated_time_secs(&self) -> u32 {
        match self {
            Self::ShapE => 45,  // ~30-60 seconds
        }
    }

    /// Quality tier
    pub fn quality(&self) -> Quality {
        match self {
            Self::ShapE => Quality::High,
        }
    }

    /// All available models
    pub fn all() -> [Model3D; 1] {
        [Self::ShapE]
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
        Self::ShapE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_ids() {
        assert_eq!(Model3D::ShapE.id(), "shap_e");
    }

    #[test]
    fn test_model_types() {
        assert_eq!(Model3D::ShapE.model_type(), ModelType::Object);
    }

    #[test]
    fn test_all_models() {
        assert_eq!(Model3D::all().len(), 1);
    }
}