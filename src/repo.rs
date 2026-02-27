use anyhow::{Context, Result};
use std::process::Command;

pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
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

pub fn get_latest_commit() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
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

pub fn get_repo_info() -> Result<(String, String)> {
    let remotes = ["origin", "upstream"];
    let mut url = String::new();

    for remote in remotes {
        let output = Command::new("git")
            .args(["remote", "get-url", remote])
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

    let parts: Vec<&str> = if url.contains("github.com") {
        if url.starts_with("git@") {
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
