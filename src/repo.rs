use anyhow::{Context, Result};
use std::process::Command;

fn parse_github_owner_repo(url: &str) -> Option<(String, String)> {
    const PREFIXES: [&str; 4] = [
        "git@github.com:",
        "ssh://git@github.com/",
        "https://github.com/",
        "http://github.com/",
    ];

    let path = PREFIXES
        .iter()
        .find_map(|prefix| url.strip_prefix(prefix))?;
    let path = path.trim_end_matches(".git").trim_matches('/');
    let mut parts = path.split('/');

    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return None;
    }

    Some((owner.to_string(), repo.to_string()))
}

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

    for remote in remotes {
        let output = Command::new("git")
            .args(["remote", "get-url", remote])
            .output()
            .context("Failed to execute git command")?;

        if output.status.success() {
            let candidate = String::from_utf8(output.stdout)?.trim().to_string();
            if let Some((owner, repo)) = parse_github_owner_repo(&candidate) {
                return Ok((owner, repo));
            }
        }
    }

    anyhow::bail!("No GitHub remote found (tried: origin, upstream)");
}

#[cfg(test)]
mod tests {
    use super::parse_github_owner_repo;

    #[test]
    fn parse_valid_github_remote_urls() {
        let cases = [
            ("git@github.com:owner/repo.git", ("owner", "repo")),
            ("ssh://git@github.com/owner/repo.git", ("owner", "repo")),
            ("https://github.com/owner/repo.git", ("owner", "repo")),
            ("http://github.com/owner/repo", ("owner", "repo")),
            ("https://github.com/owner/repo/", ("owner", "repo")),
        ];

        for (url, expected) in cases {
            let actual = parse_github_owner_repo(url);
            assert_eq!(
                actual,
                Some((expected.0.to_string(), expected.1.to_string()))
            );
        }
    }

    #[test]
    fn reject_non_github_or_malformed_urls() {
        let invalid_cases = [
            "git@gitlab.com:owner/repo.git",
            "https://example.com/owner/repo.git",
            "git@github.com:owner.git",
            "https://github.com/owner",
            "https://github.com/owner/repo/extra",
            "https://github.com//repo.git",
            "https://github.com/owner/.git",
        ];

        for url in invalid_cases {
            assert_eq!(parse_github_owner_repo(url), None);
        }
    }
}
