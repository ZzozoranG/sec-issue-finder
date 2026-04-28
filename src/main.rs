use anyhow::Result;
use clap::Parser;
use sec_issue_finder::cli::{Cli, Command};
use sec_issue_finder::clients::osv::OsvClient;
use sec_issue_finder::{policy, reporters, scan};
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Scan(args) => {
            debug!(?args, "starting scan");
            let config = args.into_scan_config();
            let format = config.format;
            let fail_on = config.fail_on;
            let osv_client = OsvClient::new();
            let result = scan::scan(config, &osv_client).await?;
            reporters::print_with_format(&result, format)?;
            policy::evaluate_with_threshold(&result, fail_on)?;
        }
    }

    Ok(())
}
