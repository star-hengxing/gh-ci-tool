mod args;
mod logs;
mod models;
mod output;
mod repo;
mod report;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use logs::{download_job_log, sanitize_path_component};
use models::{JobStatus, WorkflowStatus, is_failed_job};
use octocrab::Octocrab;
use output::OutputMode;
use repo::{get_current_branch, get_latest_commit, get_repo_info};
use report::{render_human_report, render_llm_report};
use std::fs;
use std::path::PathBuf;

fn env_truthy(key: &str) -> bool {
    matches!(
        std::env::var(key)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

async fn fetch_current_branch_workflows(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    branch: &str,
    commit_sha: &str,
) -> Result<Vec<WorkflowStatus>> {
    let workflow_runs = octocrab
        .workflows(owner, repo)
        .list_all_runs()
        .head_sha(commit_sha)
        .branch(branch)
        .send()
        .await?;

    Ok(workflow_runs
        .items
        .into_iter()
        .map(WorkflowStatus::from_run)
        .collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let output_mode = OutputMode::new(env_truthy("LLM"));

    let token = std::env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not found. Please set it")?;

    let branch = get_current_branch()?;
    output_mode.emit_verbose(format!("Current branch: {}", branch));

    let commit_sha = get_latest_commit()?;
    output_mode.emit_verbose(format!("Latest commit: {}", commit_sha));

    let short_sha = commit_sha.get(..8).unwrap_or(commit_sha.as_str());
    let output_dir = PathBuf::from(".gh-ci-tool").join(short_sha);
    fs::create_dir_all(&output_dir)?;

    let (owner, repo) = get_repo_info()?;
    output_mode.emit_verbose(format!("Repository: {}/{}", owner, repo));

    let octocrab = Octocrab::builder().personal_token(token).build()?;

    let workflow_status_json_path = output_dir.join("ci-status.json");
    let mut workflow_statuses: Vec<WorkflowStatus> = if workflow_status_json_path.exists() {
        let cached_workflow_statuses: Vec<WorkflowStatus> = serde_json::from_str(
            &fs::read_to_string(&workflow_status_json_path)
                .context("Failed to read existing ci-status.json")?,
        )
        .context("Failed to parse existing ci-status.json")?;

        if cached_workflow_statuses.is_empty() {
            output_mode.emit_verbose("Cached ci-status.json is empty; refreshing workflows...");
            fetch_current_branch_workflows(&octocrab, &owner, &repo, &branch, &commit_sha).await?
        } else {
            cached_workflow_statuses
        }
    } else {
        output_mode.emit_verbose("Fetching current branch workflows...");
        fetch_current_branch_workflows(&octocrab, &owner, &repo, &branch, &commit_sha).await?
    };

    let fetch_targets = workflow_statuses
        .iter()
        .filter(|status| status.needs_job_refresh())
        .count();
    let progress = output_mode.progress_bar(fetch_targets as u64);

    for status in &mut workflow_statuses {
        if !status.needs_job_refresh() {
            continue;
        }

        if let Some(progress) = &progress {
            progress.set_message(format!("Fetching {} jobs", status.name));
        }

        let jobs_response = octocrab
            .workflows(&owner, &repo)
            .list_jobs(status.run_id.into())
            .send()
            .await?;

        status.jobs = jobs_response
            .items
            .into_iter()
            .map(JobStatus::from_job)
            .collect();

        if let Some(progress) = &progress {
            progress.inc(1);
        }
    }

    if let Some(progress) = &progress {
        progress.finish_and_clear();
    }

    let output = serde_json::to_string(&workflow_statuses)?;
    fs::write(&workflow_status_json_path, &output)?;

    let human_report = render_human_report(&workflow_statuses);
    let llm_report = render_llm_report(&workflow_statuses);

    output_mode.emit_report(if output_mode.is_llm() {
        &llm_report
    } else {
        &human_report
    });

    let status_plain_path = output_dir.join("ci-status.txt");
    let status_llm_path = output_dir.join("ci-status.llm.txt");

    fs::write(&status_plain_path, &human_report)?;
    if output_mode.is_llm() {
        fs::write(&status_llm_path, &llm_report)?;
    }

    if !args.no_logs {
        let logs_root = output_dir.join("logs");

        for status in &workflow_statuses {
            if status.is_success() {
                continue;
            }

            let failed_jobs = status
                .jobs
                .iter()
                .filter(|job| is_failed_job(job))
                .collect::<Vec<_>>();
            if failed_jobs.is_empty() {
                continue;
            }

            let logs_path = logs_root.join(format!(
                "{}-{}",
                sanitize_path_component(&status.name),
                status.run_id
            ));
            fs::create_dir_all(&logs_path)?;

            for job in failed_jobs {
                if job.job_id == 0 {
                    continue;
                }

                let file_name =
                    format!("{}-{}.log", sanitize_path_component(&job.name), job.job_id);
                let output_path = logs_path.join(&file_name);
                if !output_path.exists() {
                    let bytes = download_job_log(&octocrab, &owner, &repo, job.job_id).await?;
                    fs::write(&output_path, &bytes).with_context(|| {
                        format!(
                            "Failed to write failed job log to {}",
                            output_path.display()
                        )
                    })?;
                }

                let log_saved_message =
                    format!("Failed job log saved to {}", output_path.display());
                output_mode.emit_report(&log_saved_message);
            }
        }
    }

    Ok(())
}
