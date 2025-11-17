// crates/gj-app/src/lgm_worker.rs

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{self, JoinHandle};
use image::RgbaImage;
use burn::tensor::backend::Backend;
use gj_core::gaussian_cloud::GaussianCloud;
use gj_lgm::LGMPipeline;

pub enum WorkerCommand {
    GenerateFromImages(Vec<RgbaImage>),
    GenerateFromPrompt(String),
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
    pub fn new<B: Backend + 'static>(device: burn::tensor::Device<B>) -> Self
    where
        B::FloatTensorPrimitive: Send,
        B::IntTensorPrimitive: Send,
    {
        let (cmd_tx, cmd_rx) = channel::<WorkerCommand>();
        let (resp_tx, resp_rx) = channel::<WorkerResponse>();

        let thread_handle = thread::spawn(move || {
            // Create pipeline in the worker thread
            let pipeline: LGMPipeline<B> = LGMPipeline::new(device);

            // Worker loop
            loop {
                match cmd_rx.recv() {
                    Ok(WorkerCommand::GenerateFromImages(images)) => {
                        let _ = resp_tx.send(WorkerResponse::Status("Processing images...".into()));

                        match pipeline.generate(&images) {
                            Ok(cloud) => {
                                let _ = resp_tx.send(WorkerResponse::Status(
                                    format!("Generated {} Gaussians", cloud.count)
                                ));
                                let _ = resp_tx.send(WorkerResponse::Success(cloud));
                            }
                            Err(e) => {
                                let _ = resp_tx.send(WorkerResponse::Error(e.to_string()));
                            }
                        }
                    }
                    Ok(WorkerCommand::GenerateFromPrompt(prompt)) => {
                        let _ = resp_tx.send(WorkerResponse::Status(
                            format!("Generating images from: '{}'", prompt)
                        ));

                        // Generate multi-view images from prompt
                        match gj_lgm::text_to_image::generate_multiview_from_prompt(&prompt) {
                            Ok(images) => {
                                let _ = resp_tx.send(WorkerResponse::Status("Running 3D generation...".into()));

                                match pipeline.generate(&images) {
                                    Ok(cloud) => {
                                        let _ = resp_tx.send(WorkerResponse::Status(
                                            format!("Generated {} Gaussians from prompt", cloud.count)
                                        ));
                                        let _ = resp_tx.send(WorkerResponse::Success(cloud));
                                    }
                                    Err(e) => {
                                        let _ = resp_tx.send(WorkerResponse::Error(e.to_string()));
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = resp_tx.send(WorkerResponse::Error(
                                    format!("Image generation failed: {}", e)
                                ));
                            }
                        }
                    }
                    Ok(WorkerCommand::Shutdown) => {
                        break;
                    }
                    Err(_) => {
                        // Channel closed, exit
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

    pub fn send_prompt(&self, prompt: String) -> Result<(), String> {
        self.command_tx
            .send(WorkerCommand::GenerateFromPrompt(prompt))
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