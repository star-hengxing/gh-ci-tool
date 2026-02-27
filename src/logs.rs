use anyhow::{Context, Result};
use http_body_util::BodyExt;
use octocrab::Octocrab;

pub fn sanitize_path_component(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_dash = false;

    for ch in name.chars() {
        let mapped = if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            ch
        } else {
            '-'
        };

        if mapped == '-' {
            if !last_dash {
                out.push('-');
                last_dash = true;
            }
        } else {
            out.push(mapped);
            last_dash = false;
        }
    }

    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_string()
    }
}

pub async fn download_job_log(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    job_id: u64,
) -> Result<Vec<u8>> {
    let route = format!("/repos/{owner}/{repo}/actions/jobs/{job_id}/logs");
    let response = octocrab
        ._get(route)
        .await
        .with_context(|| format!("Failed to request logs for job {}", job_id))?;
    let response = octocrab
        .follow_location_to_data(response)
        .await
        .with_context(|| format!("Failed to follow logs redirect for job {}", job_id))?;
    let bytes = response
        .into_body()
        .collect()
        .await
        .with_context(|| format!("Failed to download logs body for job {}", job_id))?
        .to_bytes();
    Ok(bytes.to_vec())
}
