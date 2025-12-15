mod config;
mod routes;
mod state;
mod schemas;

use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread;
use axum::Router;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use crate::error::AppError;
use crate::generator::backend::config::GenBackendConfig;
use crate::generator::backend::routes::api_routes;
use crate::generator::backend::state::GenState;
use crate::generator::{GeneratorCommand, GeneratorResponse};
use crate::generator::backend::schemas::{GenerationJob, JobResponse, JobStatusResponse};

#[derive(Deserialize)]
struct JobResult {
    output_path: String,
    model: String,
    prompt: String,
}

pub struct GenBackend {
    config: GenBackendConfig,
    status_rx: Receiver<JobStatusResponse>,
}

impl GenBackend {
    pub async fn new() -> anyhow::Result<Self> {
        let conf = GenBackendConfig::load()?;

        let state = GenState::new();
        let status_rx = state.status_rx();

        let app = Router::new()
            .merge(api_routes())
            .with_state(Arc::new(state));

        let addr = SocketAddr::from(([0, 0, 0, 0], conf.port));

        // Start the server
        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(Self {
            config: conf,
            status_rx
        })
    }

    pub fn submit_job(&self, prompt: &str, model: &gj_core::Model3D) -> anyhow::Result<()> {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://127.0.0.1:{}/generate", self.config.port);

        let request_body = GenerationJob {
            prompt: prompt.to_string(),
            model: model.id().to_string(),
            guidance_scale: 15.0,
            num_inference_steps: 64,
        };

        let response = client
            .post(url)
            .json(&request_body)
            .send()
            .map_err(|e| format!("Failed to connect: {}. Make sure FastAPI service is running (cd python && docker-compose up)", e))?;

        if !response.status().is_success() {
            return Err(anyhow::Error::from(AppError::BackendError(String::from(response.status().as_str()))));
        }

        let result: JobResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(())
    }

}