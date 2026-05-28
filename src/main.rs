use anyhow::Result;
use clap::Parser;
use claude_code_manager::cli::{Cli, Commands};
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
                .unwrap_or_else(|_| "claude_code_manager=debug,tower_http=info".into()),
        )
        .init();

    info!("Starting ccm v{}", claude_code_manager::VERSION);

    let cli = Cli::parse();

    match cli.command {
        Commands::Info { verbose } => {
            if verbose {
                println!("Name:        {}", claude_code_manager::NAME);
                println!("Version:     {}", claude_code_manager::VERSION);
                println!("Features:    CLI · API · Logging · Error handling");
                println!("Repository:  https://github.com/RajaRakoto/ccm");
            } else {
                println!("ccm v{}", claude_code_manager::VERSION);
            }
        }

        Commands::Run { mode } => {
            info!("Running in '{}' mode", mode);
            println!("Running in {} mode", mode);
            // 👉 Add your business logic here
        }

        Commands::Api { port } => {
            info!("Starting API server on port {}", port);
            claude_code_manager::api::start_server(port).await?;
        }

        Commands::Install => {
            claude_code_manager::commands::runInstall()?;
        }

        Commands::Remove => {
            claude_code_manager::commands::runRemove()?;
        }

        Commands::Reinstall => {
            claude_code_manager::commands::runReinstall()?;
        }

        Commands::Backup => {
            claude_code_manager::commands::runBackup()?;
        }

        Commands::Restore => {
            claude_code_manager::commands::runRestore()?;
        }

        Commands::Status => {
            claude_code_manager::commands::runStatus()?;
        }

        Commands::Health => {
            claude_code_manager::commands::runHealth()?;
        }

        Commands::Update => {
            claude_code_manager::commands::runUpdate()?;
        }

        Commands::Gen => {
            claude_code_manager::commands::runGen()?;
        }

        Commands::Init => {
            claude_code_manager::commands::runInit()?;
        }
    }

    Ok(())
}
