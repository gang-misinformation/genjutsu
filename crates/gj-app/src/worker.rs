use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{self, JoinHandle};
use image::RgbaImage;
use gj_core::gaussian_cloud::GaussianCloud;
use gj_core::Model3D;

pub enum WorkerCommand {
    GenerateFromImages(Vec<RgbaImage>),
    GenerateFromPrompt { prompt: String, model: Model3D },
    Shutdown,
}

pub enum WorkerResponse {
    Success(GaussianCloud),
    Error(String),
    Progress(f32),
    Status(String),
}

pub struct LGMWorker {
    pub(crate) command_tx: Sender<WorkerCommand>,
    pub(crate) response_rx: Receiver<WorkerResponse>,
    thread_handle: Option<JoinHandle<()>>,
}

impl LGMWorker {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = channel::<WorkerCommand>();
        let (resp_tx, resp_rx) = channel::<WorkerResponse>();

        let thread_handle = thread::spawn(move || {
            // Worker loop
            loop {
                match cmd_rx.recv() {
                    Ok(WorkerCommand::GenerateFromImages(images)) => {
                        let _ = resp_tx.send(WorkerResponse::Status("Processing images...".into()));
                        let _ = resp_tx.send(WorkerResponse::Error(
                            "Image-based generation not yet implemented with Shap-E. Use text prompts instead.".into()
                        ));
                    }

                    Ok(WorkerCommand::GenerateFromPrompt { prompt, model }) => {
                        let _ = resp_tx.send(WorkerResponse::Status(
                            format!("Generating with {} from: '{}'", model.name(), prompt)
                        ));

                        // Call Shap-E service
                        match generate_with_shap_e(&prompt, model) {
                            Ok(ply_path) => {
                                let _ = resp_tx.send(WorkerResponse::Status("Loading generated Gaussians...".into()));

                                match gj_core::gaussian_cloud::GaussianCloud::from_ply(&ply_path) {
                                    Ok(cloud) => {
                                        let _ = resp_tx.send(WorkerResponse::Status(
                                            format!("Loaded {} Gaussians", cloud.count)
                                        ));
                                        let _ = resp_tx.send(WorkerResponse::Success(cloud));
                                    }
                                    Err(e) => {
                                        let _ = resp_tx.send(WorkerResponse::Error(
                                            format!("Failed to load .ply: {}", e)
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = resp_tx.send(WorkerResponse::Error(
                                    format!("Generation failed: {}", e)
                                ));
                            }
                        }
                    }

                    Ok(WorkerCommand::Shutdown) => {
                        break;
                    }

                    Err(_) => {
                        break;
                    }
                }
            }
        });

        Self {
            command_tx: cmd_tx,
            response_rx: resp_rx,
            thread_handle: Some(thread_handle),
        }
    }

    pub fn send_images(&self, images: Vec<RgbaImage>) -> Result<(), String> {
        self.command_tx
            .send(WorkerCommand::GenerateFromImages(images))
            .map_err(|e| format!("Failed to send images to worker: {}", e))
    }

    pub fn send_prompt(&self, prompt: String, model: Model3D) -> Result<(), String> {
        self.command_tx
            .send(WorkerCommand::GenerateFromPrompt { prompt, model })
            .map_err(|e| format!("Failed to send prompt to worker: {}", e))
    }

    pub fn try_recv_response(&self) -> Option<WorkerResponse> {
        self.response_rx.try_recv().ok()
    }

    pub fn shutdown(&mut self) {
        let _ = self.command_tx.send(WorkerCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for LGMWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Generate with Shap-E using the Python service
fn generate_with_shap_e(prompt: &str, model: Model3D) -> Result<std::path::PathBuf, String> {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    struct GenerateRequest {
        prompt: String,
        model: String,
        guidance_scale: f32,
        num_inference_steps: usize,
    }

    #[derive(Deserialize)]
    struct GenerateResponse {
        status: String,
        output_path: Option<String>,
        error: Option<String>,
    }

    let client = reqwest::blocking::Client::new();
    let url = "http://127.0.0.1:5000/generate";

    // Shap-E works best with these parameters
    let request_body = GenerateRequest {
        prompt: prompt.to_string(),
        model: "shap_e".to_string(),  // Always use Shap-E
        guidance_scale: 15.0,  // Shap-E recommended value
        num_inference_steps: 64,  // Default for quality/speed balance
    };

    let response = client
        .post(url)
        .json(&request_body)
        .send()
        .map_err(|e| format!("Failed to connect to service: {}. Make sure Python service is running (conda activate genjutsu && cd python && python multi_model_service.py)", e))?;

    if !response.status().is_success() {
        return Err(format!("Service returned error: {}", response.status()));
    }

    let result: GenerateResponse = response
        .json()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    match result.status.as_str() {
        "success" => {
            let output_path = result.output_path
                .ok_or_else(|| "No output path returned".to_string())?;

            // Convert service path to host path
            let host_path = if output_path.starts_with("/app/outputs/") {
                std::path::PathBuf::from(output_path.replace("/app/outputs/", "outputs/"))
            } else if output_path.starts_with("../outputs/") {
                std::path::PathBuf::from(output_path.replace("../outputs/", "outputs/"))
            } else if output_path.starts_with("outputs/") {
                std::path::PathBuf::from(output_path)
            } else {
                let filename = std::path::Path::new(&output_path)
                    .file_name()
                    .ok_or_else(|| "Invalid output path".to_string())?;
                std::path::PathBuf::from("outputs").join(filename)
            };

            if !host_path.exists() {
                return Err(format!("Generated file not found at: {}", host_path.display()));
            }

            Ok(host_path)
        }
        "error" => {
            let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
            Err(format!("Service error: {}", error_msg))
        }
        _ => {
            Err(format!("Unexpected status: {}", result.status))
        }
    }
}