mod app;
mod cli;
mod event;
mod kafka;
mod state;
mod ui;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = cli::Cli::parse();
    let mut app = app::App::new(cli);
    app.run().await
}
