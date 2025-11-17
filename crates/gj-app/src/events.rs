#[derive(Debug, Clone)]
pub enum GjEvent {
    Ui(UiEvent),
    App(AppEvent)
}


#[derive(Debug, Clone)]
pub enum UiEvent {
    ResetCamera,
    LoadImages,
    GenerateFromPrompt(String),
    PromptChanged(String),
    ToggleWireframe(bool),
    Log(String),
}


#[derive(Debug, Clone)]
pub enum AppEvent {
    ImagesLoaded,
    GaussianCloudReady,
    CameraResetDone,
    Status(String),
    Progress(f32),
    Log(String),
    WireframeState(bool),
    SceneReady
}