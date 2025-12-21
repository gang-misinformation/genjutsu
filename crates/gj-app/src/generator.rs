use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use image::RgbaImage;
use gj_core::gaussian_cloud::GaussianCloud;
use gj_core::Model3D;
use crate::backend::GenBackend;
use crate::db::job::JobRecord;

pub enum GeneratorCommand {
    GenerateFromImages(Vec<RgbaImage>),
    GenerateFromPrompt { prompt: String, model: Model3D },
    Shutdown,
}

pub enum GeneratorResponse {
    Success(GaussianCloud),
    Error(String),
    Progress(String, f32, String),
    Status(String),
    JobSubmitted(JobRecord)
}

pub struct Generator {
    pub(crate) command_tx: Sender<GeneratorCommand>,
    pub(crate) response_rx: Receiver<GeneratorResponse>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Generator {
    pub async fn new() -> anyhow::Result<Self> {
        let (cmd_tx, cmd_rx) = channel::<GeneratorCommand>();
        let (resp_tx, resp_rx) = channel::<GeneratorResponse>();
        
        
        let thread_handle = thread::spawn(move || {
            // Generator loop
            loop {
                match cmd_rx.recv() {
                    Ok(GeneratorCommand::GenerateFromImages(images)) => {
                        let _ = resp_tx.send(GeneratorResponse::Status("Processing images...".into()));
                        let _ = resp_tx.send(GeneratorResponse::Error(
                            "Image-based generation not yet implemented with Shap-E. Use text prompts instead.".into()
                        ));
                    }

                    Ok(GeneratorCommand::GenerateFromPrompt { prompt, model }) => {
                        let _ = resp_tx.send(GeneratorResponse::Status(
                            format!("Submitting job to {} service...", model.name())
                        ));

                        let _ = resp_tx.send(GeneratorResponse::JobSubmitted);
                        let _ = resp_tx.send(GeneratorResponse::Status(
                            String::from("Job submitted")
                        ));
                    }

                    Ok(GeneratorCommand::Shutdown) => {
                        break;
                    }

                    Err(_) => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            command_tx: cmd_tx,
            response_rx: resp_rx,
            thread_handle: Some(thread_handle),
        })
    }

    pub fn try_recv_response(&self) -> Option<GeneratorResponse> {
        self.response_rx.try_recv().ok()
    }

    fn shutdown(&mut self) {
        let _ = self.command_tx.send(GeneratorCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Generator {
    fn drop(&mut self) {
        self.shutdown();
    }
}