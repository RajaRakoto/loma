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

    loma::utils::banner::showBanner();

    let cli = match Cli::try_parse() {
        Ok(parsed) => parsed,
        Err(err) => {
            use clap::error::ErrorKind;
            match err.kind() {
                ErrorKind::DisplayHelp => {
                    loma::utils::banner::showHelp();
                    std::process::exit(0);
                }
                ErrorKind::DisplayVersion => {
                    println!("loma v{}", loma::VERSION);
                    std::process::exit(0);
                }
                _ => {
                    let err_msg = err.to_string();
                    let first_line = err_msg.lines().next().unwrap_or("");
                    eprintln!("\x1b[31;1mError:\x1b[0m {}", first_line);
                    eprintln!();
                    loma::utils::banner::showHelp();
                    std::process::exit(1);
                }
            }
        }
    };

    if cli.version {
        println!("loma v{}", loma::VERSION);
        return Ok(());
    }

    match cli.command {
        Some(Commands::Info { verbose }) => {
            if verbose {
                println!("Name:        {}", loma::NAME);
                println!("Version:     {}", loma::VERSION);
                println!("Features:    CLI · API · Logging · Error handling");
                println!("Repository:  https://github.com/RajaRakoto/loma");
            } else {
                println!("loma v{}", loma::VERSION);
            }
        }

        Some(Commands::Run { mode }) => {
            info!("Running in '{}' mode", mode);
            println!("Running in {} mode", mode);
            // 👉 Add your business logic here
        }

        Some(Commands::Api { port }) => {
            info!("Starting API server on port {}", port);
            loma::api::start_server(port).await?;
        }

        Some(Commands::Install { assistant }) => {
            loma::commands::runInstall(&assistant)?;
        }

        Some(Commands::Remove { assistant }) => {
            loma::commands::runRemove(&assistant)?;
        }

        Some(Commands::Reinstall { assistant }) => {
            loma::commands::runReinstall(&assistant)?;
        }

        Some(Commands::Backup { assistant }) => {
            loma::commands::runBackup(&assistant)?;
        }

        Some(Commands::Restore { assistant }) => {
            loma::commands::runRestore(&assistant)?;
        }

        Some(Commands::Status { assistant }) => {
            loma::commands::runStatus(&assistant)?;
        }

        Some(Commands::Health) => {
            loma::commands::runHealth()?;
        }

        Some(Commands::Update { assistant }) => {
            loma::commands::runUpdate(&assistant)?;
        }

        Some(Commands::Optimize { assistant }) => {
            loma::commands::runOptimize(&assistant)?;
        }

        Some(Commands::Gen { assistant }) => {
            loma::commands::runGen(&assistant)?;
        }

        Some(Commands::Init { assistant }) => {
            loma::commands::runInit(&assistant)?;
        }

        None => {
            loma::utils::banner::showHelp();
        }
    }

    Ok(())
}
