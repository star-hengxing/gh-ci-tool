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

pub fn render_human_report(workflows: &[WorkflowStatus]) -> String {
    let mut lines = Vec::new();

    for workflow in workflows {
        match workflow.conclusion.as_deref() {
            Some("success") => lines.push(format!("- {} {}", workflow.name, icon_success())),
            Some("failure") => {
                lines.push(format!("- {} {}", workflow.name, icon_failure()));
                for job in &workflow.jobs {
                    lines.push(format!("  - {}: {}", job.name, format_job_conclusion(job)));
                }
            }
            _ => lines.push(format!(
                "- {} status={} conclusion={}",
                workflow.name,
                workflow.status,
                workflow.conclusion.as_deref().unwrap_or("unknown")
            )),
        }
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

    if failure == 0 {
        lines.push("all workflows passed".to_string());
    }

    lines.join("\n")
}
