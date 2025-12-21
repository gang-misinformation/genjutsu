use std::sync::mpsc::{channel, Receiver, Sender};
use crate::backend::schemas::{JobStatusPair, JobStatusResponse};

pub struct GenState {
    status_tx: Sender<JobStatusPair>,
}

impl GenState {
    pub fn new(status_tx: Sender<JobStatusPair>) -> Self {
        Self {
            status_tx
        }
    }

    pub fn emit_job_status(&self, id: String, resp: JobStatusResponse) -> anyhow::Result<()> {
        self.status_tx.send((id, resp))?;
        Ok(())
    }
}