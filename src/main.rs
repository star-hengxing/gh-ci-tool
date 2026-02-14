use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Cursor, IsTerminal};
use std::path::PathBuf;
use std::process::Command;
use zip::ZipArchive;

#[derive(Parser, Debug)]
#[command(name = "gh-ci-tool")]
#[command(about = "Check GitHub Actions CI status for current commit")]
struct Args {
    /// Disable log download for failed workflows
    #[arg(long, default_value_t = false)]
    no_logs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JobStatus {
    name: String,
    status: octocrab::models::workflows::Status,
    conclusion: Option<octocrab::models::workflows::Conclusion>,
    html_url: String,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowStatus {
    #[serde(default)]
    run_id: u64,
    name: String,
    status: String,
    conclusion: Option<String>,
    html_url: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    jobs: Vec<JobStatus>,
}

fn supports_emoji() -> bool {
    // Heuristic: if stdout is a terminal and user didn't force disable.
    // Windows Terminal / modern terminals generally support emoji.
    std::io::stdout().is_terminal() && std::env::var("NO_EMOJI").is_err()
}

fn icon_success() -> &'static str {
    if supports_emoji() { "✔" } else { "✓" }
}

fn icon_failure() -> &'static str {
    if supports_emoji() { "❌" } else { "✗" }
}

fn icon_pending() -> &'static str {
    if supports_emoji() { "⏳" } else { "~" }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let token = std::env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not found. Please set it")?;

    let branch = get_current_branch()?;
    println!("Current branch: {}", branch);

    let commit_sha = get_latest_commit()?;
    println!("Latest commit: {}", commit_sha);

    let output_dir = PathBuf::from(".gh-ci-tool").join(&commit_sha[..8]);
    fs::create_dir_all(&output_dir)?;

    let (owner, repo) = get_repo_info()?;
    println!("Repository: {}/{}", owner, repo);

    let octocrab = Octocrab::builder().personal_token(token).build()?;

    let workflow_status_json_path = output_dir.join("ci-status.json");
    let mut workflow_statuses: Vec<WorkflowStatus> = if workflow_status_json_path.exists() {
        serde_json::from_str(
            &fs::read_to_string(&workflow_status_json_path)
                .context("Failed to read existing ci-status.json")?,
        )
        .context("Failed to parse existing ci-status.json")?
    } else {
        println!("Fetching current branch workflows...");
        let workflow_runs = octocrab
            .workflows(&owner, &repo)
            .list_all_runs()
            .head_sha(&commit_sha)
            .branch(&branch)
            .send()
            .await?;

        workflow_runs
            .items
            .into_iter()
            .map(|run| WorkflowStatus {
                run_id: run.id.0,
                name: run.name.clone(),
                status: run.status.to_string(),
                conclusion: run.conclusion,
                html_url: run.html_url.to_string(),
                created_at: run.created_at,
                updated_at: run.updated_at,
                jobs: Vec::new(),
            })
            .collect()
    };

    let progress = if std::io::stdout().is_terminal() {
        let progress = ProgressBar::new(workflow_statuses.len() as u64);
        progress.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {pos}/{len} {wide_msg}",
            )
            .unwrap(),
        );
        Some(progress)
    } else {
        None
    };

    for status in &mut workflow_statuses {
        if status.conclusion.as_deref() == Some("success") {
            continue;
        }
        if !status.jobs.is_empty() {
            use octocrab::models::workflows::Status::*;
            let need_fetch = status
                .jobs
                .iter()
                .all(|j| j.status != Completed && j.status != Failed);
            if !need_fetch {
                continue;
            }
        }

        status.jobs = {
            if let Some(progress) = &progress {
                progress.set_message(format!("Fetching {} jobs", status.name));
            }

            let jobs_response = octocrab
                .workflows(&owner, &repo)
                .list_jobs(status.run_id.into())
                .send()
                .await?;

            jobs_response
                .items
                .into_iter()
                .map(|job| JobStatus {
                    name: job.name,
                    status: job.status,
                    conclusion: job.conclusion,
                    html_url: job.html_url.to_string(),
                    started_at: job.started_at,
                    completed_at: job.completed_at,
                })
                .collect()
        };

        if let Some(progress) = &progress {
            progress.inc(1);
        }
    }

    let output = serde_json::to_string_pretty(&workflow_statuses)?;
    fs::write(&workflow_status_json_path, &output)?;

    let mut status_string = Vec::new();
    for status in &workflow_statuses {
        match status.conclusion.as_deref() {
            Some("success") => status_string.push(format!("- {} {}", status.name, icon_success())),
            Some("failure") => {
                status_string.push(format!("- {} {}", status.name, icon_failure()));
                for job in &status.jobs {
                    status_string.push(format!(
                        "  - {}: {}",
                        job.name,
                        job.conclusion
                            .as_ref()
                            .map(|s| format!("{:?}", s))
                            .unwrap_or_else(|| "unknown".to_string())
                    ));
                }
            }
            _ => status_string.push(format!("- {} unknown status", status.name)),
        }
    }

    let output = status_string.join("\n");
    println!("{}", output);

    let status_plain_path = output_dir.join("ci-status.txt");
    fs::write(&status_plain_path, &output)?;

    if !args.no_logs {
        for status in &workflow_statuses {
            if status.conclusion.as_deref() == Some("success") {
                continue;
            }

            let logs_path = output_dir.join("logs").join(format!(
                "{}-{}",
                status.name.replace(" ", "-"),
                status.run_id
            ));
            if logs_path.exists() {
                continue;
            } else {
                fs::create_dir_all(&logs_path)?;
            }

            download_workflow_logs(&octocrab, &owner, &repo, status.run_id, &logs_path).await?;
        }
    }

    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn get_latest_commit() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn get_repo_info() -> Result<(String, String)> {
    let remotes = ["origin", "upstream"];
    let mut url = String::new();

    for remote in remotes {
        let output = Command::new("git")
            .args(&["remote", "get-url", remote])
            .output()
            .context("Failed to execute git command")?;

        if output.status.success() {
            url = String::from_utf8(output.stdout)?.trim().to_string();
            if url.contains("github.com") {
                break;
            }
        }
    }

    if url.is_empty() {
        anyhow::bail!("No GitHub remote found (tried: origin, upstream)");
    }

    // Parse GitHub URL (supports both HTTPS and SSH)
    let parts: Vec<&str> = if url.contains("github.com") {
        if url.starts_with("git@") {
            // SSH format: git@github.com:owner/repo.git
            url.split("github.com:")
                .nth(1)
                .context("Invalid SSH URL format")?
                .trim_end_matches(".git")
                .split('/')
                .collect()
        } else {
            url.split("github.com/")
                .nth(1)
                .context("Invalid HTTPS URL format")?
                .trim_end_matches(".git")
                .split('/')
                .collect()
        }
    } else {
        anyhow::bail!("Not a GitHub repository");
    };

    if parts.len() < 2 {
        anyhow::bail!("Could not parse owner/repo from URL");
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

async fn download_workflow_logs(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    run_id: u64,
    output_dir: &PathBuf,
) -> Result<()> {
    let bytes = octocrab
        .actions()
        .download_workflow_run_logs(owner, repo, run_id.into())
        .await?;

    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).context("Invalid zip archive from GitHub")?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if !entry.is_file() {
            continue;
        }

        let entry_name = entry.name();
        let Some(file_name) = entry_name.rsplit('/').next() else {
            continue;
        };
        if file_name.is_empty() {
            continue;
        }

        let output_path = output_dir.join(file_name);
        let mut out = File::create(&output_path)?;
        io::copy(&mut entry, &mut out)?;
    }

    println!("Logs extracted to {}", output_dir.display());

    Ok(())
}
