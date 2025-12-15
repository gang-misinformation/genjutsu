use std::sync::mpsc::{channel, Receiver, Sender};
use crate::generator::backend::schemas::JobStatusResponse;

pub struct GenState {
    status_tx: Sender<JobStatusResponse>,
    status_rx: Receiver<JobStatusResponse>,
}

impl GenState {
    pub fn new() -> Self {
        let (status_tx, status_rx) = channel::<JobStatusResponse>();
        
        Self {
            status_tx,
            status_rx
        }
    }
    
    pub fn status_rx(&self) -> Receiver<JobStatusResponse> {
        self.status_rx
    }
    
    pub fn emit_job_status(&self, resp: JobStatusResponse) -> anyhow::Result<()> {
        self.status_tx.send(resp)?;
        Ok(())
    }
}