use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gh-ci-tool")]
#[command(about = "Check GitHub Actions CI status for current commit")]
#[command(version)]
pub struct Args {
    /// Disable log download for failed jobs
    #[arg(long, default_value_t = false)]
    pub no_logs: bool,
}
