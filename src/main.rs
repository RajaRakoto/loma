use anyhow::Result;
use clap::Parser;
use rust_boilerplate::cli::{Cli, Commands};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present (silently ignored if missing)
    dotenvy::dotenv().ok();

    // Initialize tracing from RUST_LOG env var,
    // defaulting to debug for this crate and info for everything else
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_boilerplate=debug,tower_http=info".into()),
        )
        .init();

    info!("Starting rust-boilerplate v{}", rust_boilerplate::VERSION);

    let cli = Cli::parse();

    match cli.command {
        Commands::Info { verbose } => {
            if verbose {
                println!("Name:        {}", rust_boilerplate::NAME);
                println!("Version:     {}", rust_boilerplate::VERSION);
                println!("Features:    CLI · API · Logging · Error handling");
                println!("Repository:  https://github.com/yourusername/rust-boilerplate");
            } else {
                println!("rust-boilerplate v{}", rust_boilerplate::VERSION);
            }
        }

        Commands::Run { mode } => {
            info!("Running in '{}' mode", mode);
            println!("Running in {} mode", mode);
            // 👉 Add your business logic here
        }

        Commands::Api { port } => {
            info!("Starting API server on port {}", port);
            rust_boilerplate::api::start_server(port).await?;
        }
    }

    Ok(())
}
