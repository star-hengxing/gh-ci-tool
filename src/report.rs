use crate::models::{WorkflowStatus, format_job_conclusion, is_failed_job};
use std::io::IsTerminal;

fn supports_emoji() -> bool {
    std::io::stdout().is_terminal() && std::env::var("NO_EMOJI").is_err()
}

fn icon_success() -> &'static str {
    if supports_emoji() { "\u{2705}" } else { "[OK]" }
}

fn icon_failure() -> &'static str {
    if supports_emoji() {
        "\u{274C}"
    } else {
        "[FAIL]"
    }
}

fn icon_running() -> &'static str {
    if supports_emoji() {
        "\u{23F3}"
    } else {
        "[RUNNING]"
    }
}

pub fn render_human_report(workflows: &[WorkflowStatus]) -> String {
    let mut lines = Vec::new();

    for workflow in workflows {
        if workflow.is_success() {
            lines.push(format!("- {} {}", workflow.name, icon_success()));
            continue;
        }

        if workflow.is_failure() {
            lines.push(format!("- {} {}", workflow.name, icon_failure()));
            for job in &workflow.jobs {
                lines.push(format!("  - {}: {}", job.name, format_job_conclusion(job)));
            }
            continue;
        }

        if workflow.is_in_progress() {
            lines.push(format!(
                "- {} {} ({})",
                workflow.name,
                icon_running(),
                workflow.status
            ));
            continue;
        }

        lines.push(format!(
            "- {} status={} conclusion={}",
            workflow.name,
            workflow.status,
            workflow.conclusion.as_deref().unwrap_or("unknown")
        ));
    }

    lines.join("\n")
}

pub fn render_llm_report(workflows: &[WorkflowStatus]) -> String {
    let total = workflows.len();
    let success = workflows.iter().filter(|w| w.is_success()).count();
    let failure = workflows.iter().filter(|w| w.is_failure()).count();
    let other = total.saturating_sub(success + failure);

    let mut lines = vec![format!(
        "ci total={} success={} failure={} other={}",
        total, success, failure, other
    )];

    for workflow in workflows.iter().filter(|w| w.is_failure()) {
        let failed_jobs = workflow
            .jobs
            .iter()
            .filter(|job| is_failed_job(job))
            .collect::<Vec<_>>();

        lines.push(format!(
            "fail workflow={} run_id={} failed_jobs={}",
            workflow.name,
            workflow.run_id,
            failed_jobs.len()
        ));

        for job in failed_jobs {
            lines.push(format!(
                "  - job={} conclusion={}",
                job.name,
                format_job_conclusion(job)
            ));
        }
    }

    for workflow in workflows.iter().filter(|w| w.is_in_progress()) {
        lines.push(format!(
            "running workflow={} status={}",
            workflow.name, workflow.status
        ));
    }

    if failure == 0 && other == 0 {
        lines.push("all workflows passed".to_string());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{render_human_report, render_llm_report};
    use crate::models::WorkflowStatus;
    use chrono::Utc;

    fn workflow(name: &str, status: &str, conclusion: Option<&str>) -> WorkflowStatus {
        WorkflowStatus {
            run_id: 1,
            name: name.to_string(),
            status: status.to_string(),
            conclusion: conclusion.map(str::to_string),
            html_url: "https://example.invalid".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            jobs: Vec::new(),
        }
    }

    #[test]
    fn renders_in_progress_workflow_as_running() {
        let report = render_human_report(&[workflow("FreeBSD", "in_progress", None)]);
        assert!(report.contains("- FreeBSD [RUNNING] (in_progress)"));
    }

    #[test]
    fn llm_report_does_not_mark_running_workflows_as_passed() {
        let report = render_llm_report(&[
            workflow("Linux", "completed", Some("success")),
            workflow("FreeBSD", "in_progress", None),
        ]);

        assert!(!report.contains("all workflows passed"));
        assert!(report.contains("running workflow=FreeBSD status=in_progress"));
    }

    #[test]
    fn llm_report_marks_all_passed_only_when_no_other_states() {
        let report = render_llm_report(&[
            workflow("Linux", "completed", Some("success")),
            workflow("Windows", "completed", Some("success")),
        ]);
        assert!(report.contains("all workflows passed"));
    }
}
