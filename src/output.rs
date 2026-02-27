use indicatif::{ProgressBar, ProgressStyle};
use std::io::IsTerminal;

pub struct OutputMode {
    is_llm: bool,
    show_verbose: bool,
    show_progress: bool,
}

impl OutputMode {
    pub fn new(is_llm: bool) -> Self {
        Self {
            is_llm,
            show_verbose: !is_llm,
            show_progress: !is_llm,
        }
    }

    pub fn is_llm(&self) -> bool {
        self.is_llm
    }

    pub fn emit_verbose(&self, message: impl AsRef<str>) {
        if self.show_verbose {
            println!("{}", message.as_ref());
        }
    }

    pub fn emit_report(&self, report: &str) {
        println!("{}", report);
    }

    pub fn progress_bar(&self, total: u64) -> Option<ProgressBar> {
        if !self.show_progress || !std::io::stdout().is_terminal() || total == 0 {
            return None;
        }

        let progress = ProgressBar::new(total);
        progress.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {pos}/{len} {wide_msg}",
            )
            .unwrap(),
        );
        Some(progress)
    }
}
