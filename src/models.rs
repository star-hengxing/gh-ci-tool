use chrono::{DateTime, Utc};
use octocrab::models::workflows::{Conclusion, Job, Run, Status};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    #[serde(default)]
    pub job_id: u64,
    pub name: String,
    pub status: Status,
    pub conclusion: Option<Conclusion>,
    pub html_url: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl JobStatus {
    pub fn from_job(job: Job) -> Self {
        Self {
            job_id: job.id.0,
            name: job.name,
            status: job.status,
            conclusion: job.conclusion,
            html_url: job.html_url.to_string(),
            started_at: job.started_at,
            completed_at: job.completed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    #[serde(default)]
    pub run_id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub jobs: Vec<JobStatus>,
}

impl WorkflowStatus {
    pub fn from_run(run: Run) -> Self {
        Self {
            run_id: run.id.0,
            name: run.name,
            status: run.status.to_string(),
            conclusion: run.conclusion,
            html_url: run.html_url.to_string(),
            created_at: run.created_at,
            updated_at: run.updated_at,
            jobs: Vec::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.conclusion.as_deref() == Some("success")
    }

    pub fn is_failure(&self) -> bool {
        self.conclusion.as_deref() == Some("failure")
    }

    pub fn needs_job_refresh(&self) -> bool {
        if self.is_success() {
            return false;
        }
        if self.jobs.is_empty() {
            return true;
        }

        let has_terminal_jobs = self
            .jobs
            .iter()
            .any(|job| matches!(job.status, Status::Completed | Status::Failed));
        let has_missing_job_ids = self.jobs.iter().any(|job| job.job_id == 0);
        !has_terminal_jobs || has_missing_job_ids
    }
}

pub fn is_failed_job(job: &JobStatus) -> bool {
    matches!(job.status, Status::Failed) || matches!(job.conclusion, Some(Conclusion::Failure))
}

pub fn format_job_conclusion(job: &JobStatus) -> String {
    job.conclusion
        .as_ref()
        .map(|c| format!("{:?}", c))
        .unwrap_or_else(|| "Unknown".to_string())
}
