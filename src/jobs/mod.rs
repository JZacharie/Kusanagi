//! Background Jobs System
//!
//! Asynchronous job queue with retries, scheduling, and monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub mod worker;
pub use worker::JobWorker;

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Job priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        JobPriority::Normal
    }
}

/// Job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retries: u32,
    pub max_retries: u32,
    pub error_message: Option<String>,
}

impl Job {
    /// Create a new job
    pub fn new(job_type: impl Into<String>, payload: impl Serialize) -> Result<Self, serde_json::Error> {
        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            job_type: job_type.into(),
            payload: serde_json::to_value(payload)?,
            status: JobStatus::Pending,
            priority: JobPriority::Normal,
            created_at: now,
            scheduled_at: now,
            started_at: None,
            completed_at: None,
            retries: 0,
            max_retries: 3,
            error_message: None,
        })
    }

    /// Set priority
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Check if job should be retried
    pub fn should_retry(&self) -> bool {
        self.retries < self.max_retries && self.status == JobStatus::Failed
    }
}

/// Job queue
pub struct JobQueue {
    jobs: Mutex<VecDeque<Job>>,
    running: Mutex<Vec<Job>>,
    completed: Mutex<Vec<Job>>,
    tx: mpsc::Sender<Job>,
}

impl JobQueue {
    /// Create new job queue
    pub fn new() -> (Arc<Self>, mpsc::Receiver<Job>) {
        let (tx, rx) = mpsc::channel(100);
        
        let queue = Arc::new(Self {
            jobs: Mutex::new(VecDeque::new()),
            running: Mutex::new(Vec::new()),
            completed: Mutex::new(Vec::new()),
            tx,
        });

        (queue, rx)
    }

    /// Submit a job
    pub async fn submit(&self, job: Job) -> Result<String, String> {
        let id = job.id.clone();
        self.tx.send(job).await.map_err(|e| e.to_string())?;
        Ok(id)
    }

    /// Get job status
    pub fn get_job(&self, id: &str) -> Option<Job> {
        let jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.iter().find(|j| j.id == id) {
            return Some(job.clone());
        }
        drop(jobs);

        let running = self.running.lock().unwrap();
        if let Some(job) = running.iter().find(|j| j.id == id) {
            return Some(job.clone());
        }
        drop(running);

        let completed = self.completed.lock().unwrap();
        completed.iter().find(|j| j.id == id).cloned()
    }

    /// Get queue stats
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            pending: self.jobs.lock().unwrap().len(),
            running: self.running.lock().unwrap().len(),
            completed: self.completed.lock().unwrap().len(),
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
}

/// Job handler trait
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    /// Handle a job
    async fn handle(&self, job: &Job) -> Result<(), String>;
    
    /// Get job types this handler can process
    fn job_types(&self) -> Vec<&str>;
}
