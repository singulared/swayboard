use anyhow::{Context, Result};
use clap::Parser;
use swayboard::{Cli, Service};

#[tokio::main]
async fn main() -> Result<()> {
    let service = Service::init()
        .await
        .context("service initialization error")?;
    service
        .execute(&Cli::parse().command())
        .await
        .context("Command execution error")
}
