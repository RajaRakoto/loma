use anyhow::Result;
use clap::Parser;
use loma::cli::{Cli, Commands};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Load isolated loma.env first, falling back to local .env if missing
    dotenvy::from_path(".loma/loma.env").ok();
    dotenvy::dotenv().ok();

    // Initialize tracing from RUST_LOG env var,
    // defaulting to debug for this crate and info for everything else
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "loma=debug,tower_http=info".into()),
        )
        .init();

    info!("Starting loma v{}", loma::VERSION);

    let cli = Cli::parse();

    match cli.command {
        Commands::Info { verbose } => {
            if verbose {
                println!("Name:        {}", loma::NAME);
                println!("Version:     {}", loma::VERSION);
                println!("Features:    CLI · API · Logging · Error handling");
                println!("Repository:  https://github.com/RajaRakoto/loma");
            } else {
                println!("loma v{}", loma::VERSION);
            }
        }

        Commands::Run { mode } => {
            info!("Running in '{}' mode", mode);
            println!("Running in {} mode", mode);
            // 👉 Add your business logic here
        }

        Commands::Api { port } => {
            info!("Starting API server on port {}", port);
            loma::api::start_server(port).await?;
        }

        Commands::Install { assistant } => {
            loma::commands::runInstall(&assistant)?;
        }

        Commands::Remove { assistant } => {
            loma::commands::runRemove(&assistant)?;
        }

        Commands::Reinstall { assistant } => {
            loma::commands::runReinstall(&assistant)?;
        }

        Commands::Backup { assistant } => {
            loma::commands::runBackup(&assistant)?;
        }

        Commands::Restore { assistant } => {
            loma::commands::runRestore(&assistant)?;
        }

        Commands::Status { assistant } => {
            loma::commands::runStatus(&assistant)?;
        }

        Commands::Health => {
            loma::commands::runHealth()?;
        }

        Commands::Update { assistant } => {
            loma::commands::runUpdate(&assistant)?;
        }

        Commands::Optimize { assistant } => {
            loma::commands::runOptimize(&assistant)?;
        }

        Commands::Gen => {
            loma::commands::runGen()?;
        }

        Commands::Init { assistant } => {
            loma::commands::runInit(&assistant)?;
        }
    }

    Ok(())
}
