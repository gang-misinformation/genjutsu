pub mod job;

use std::path::PathBuf;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::{Notification, Surreal};
use crate::generator::db::job::{JobRecord};
use anyhow::Result;
use log::info;
use surrealdb_types::RecordId;
use thiserror::__private17::AsDisplay;
use crate::job::{Job, JobStatus};

const JOBS: &str = "jobs";

#[derive(Debug, Clone)]
pub struct JobDatabase {
    db: Surreal<Db>,
}

impl JobDatabase {
    /// Initialize SurrealDB with RocksDB backend (embedded, file-based)
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        info!("Setting up job database at {}", &db_path.as_display());

        // Create database directory
        std::fs::create_dir_all(&db_path)?;

        let db = Surreal::new::<RocksDb>(db_path).await?;
        // Use namespace and database
        db.use_ns("genjutsu").use_db("jobs").await?;

        Ok(Self { db })
    }

    /// Insert a new job
    pub async fn insert_job(&self, id: String, job: Job) -> Result<Option<JobRecord>> {
        let record: Option<JobRecord> = self.db
            .create((JOBS, id))
            .content(job)
            .await?;

        Ok(record)
    }

    /// Update job status
    pub async fn update_status(
        &self,
        job_id: String,
        status: JobStatus,
        progress: f32,
        message: Option<String>,
    ) -> Result<()> {
        let _: Option<JobRecord> = self.db
            .query("UPDATE jobs SET status = $status, progress = $progress, message = $message, updated_at = time::now() WHERE job_id = $job_id")
            .bind(("job_id", job_id))
            .bind(("status", status))
            .bind(("progress", progress))
            .bind(("message", message))
            .await?
            .take(0)?;

        Ok(())
    }

    /// Mark job as complete with result path
    pub async fn complete_job(&self, job_id: String, ply_path: PathBuf) -> Result<()> {
        let _: Option<JobRecord> = self.db
            .query("UPDATE jobs SET status = $status, progress = 1.0, ply_path = $ply_path, updated_at = time::now() WHERE job_id = $job_id")
            .bind(("job_id", job_id))
            .bind(("status", JobStatus::Complete))
            .bind(("ply_path", ply_path.to_string_lossy().to_string()))
            .await?
            .take(0)?;

        Ok(())
    }

    /// Mark job as failed
    pub async fn fail_job(&self, job_id: String, error: String) -> Result<()> {
        let _: Option<JobRecord> = self.db
            .query("UPDATE jobs SET status = $status, error = $error, updated_at = time::now() WHERE job_id = $job_id")
            .bind(("job_id", job_id))
            .bind(("status", JobStatus::Failed))
            .bind(("error", error))
            .await?
            .take(0)?;

        Ok(())
    }

    /// Get job by ID
    pub async fn get_job(&self, job_id: String) -> Result<Option<JobRecord>> {
        let mut result = self.db
            .query("SELECT * FROM jobs WHERE job_id = $job_id")
            .bind(("job_id", job_id))
            .await?;

        let jobs: Vec<JobRecord> = result.take(0)?;
        Ok(jobs.into_iter().next())
    }

    /// Get all jobs, ordered by created_at DESC
    pub async fn get_all_jobs(&self) -> Result<Vec<JobRecord>> {
        let mut result = self.db
            .query("SELECT * FROM jobs ORDER BY created_at DESC")
            .await?;

        Ok(result.take(0)?)
    }

    /// Get active jobs only
    pub async fn get_active_jobs(&self) -> Result<Vec<JobRecord>> {
        let mut result = self.db
            .query("SELECT * FROM jobs WHERE status IN ['Queued', 'Submitting', 'Generating'] ORDER BY created_at DESC")
            .await?;

        Ok(result.take(0)?)
    }

    /// Get completed jobs only
    pub async fn get_completed_jobs(&self) -> Result<Vec<JobRecord>> {
        let mut result = self.db
            .query("SELECT * FROM jobs WHERE status IN ['Complete', 'Failed'] ORDER BY created_at DESC")
            .await?;

        Ok(result.take(0)?)
    }

    /// Delete a job
    pub async fn delete_job(&self, id: RecordId) -> Result<()> {
        let _: Option<JobRecord> = self.db
            .query("DELETE FROM jobs WHERE job_id = $job_id")
            .bind(("job_id", id))
            .await?
            .take(0)?;

        Ok(())
    }

    /// Clear all completed jobs
    pub async fn clear_completed(&self) -> Result<()> {
        let _: Vec<JobRecord> = self.db
            .query("DELETE FROM jobs WHERE status IN ['Complete', 'Failed']")
            .await?
            .take(0)?;

        Ok(())
    }

    /// Subscribe to job updates (real-time)
    pub async fn subscribe_to_job_updates(&self) -> Result<impl futures::Stream<Item = JobRecord>> {
        // This is where SurrealDB shines - LIVE queries!
        use futures::StreamExt;

        let mut response = self.db
            .query("LIVE SELECT * FROM jobs")
            .await?;

        let stream = response
            .stream::<Notification<JobRecord>>(0)?;

        // Convert Notification<JobRecord> -> JobRecord
        let mapped = stream.filter_map(|notif| async move {
            match notif {
                Ok(n) => Some(n.data),
                Err(_) => None,
            }
        });

        Ok(mapped)
    }
}