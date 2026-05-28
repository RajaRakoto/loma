//! CLI interface
//!
//! Three focused commands:
//! - `info`  — print application metadata
//! - `run`   — entry point for custom business logic
//! - `api`   — start the Axum HTTP server

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ccm",
    version,
    author,
    about = ""
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available sub-commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Print application information.
    Info {
        /// Show detailed information (version, features, repo URL).
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run the application (add your business logic here).
    Run {
        /// Execution mode, e.g. `development` or `production`.
        #[arg(short, long, default_value = "development")]
        mode: String,
    },

    /// Start the Axum HTTP API server.
    Api {
        /// Port to listen on (overrides API_PORT env var).
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_info_verbose() {
        let cli = Cli::try_parse_from(["rust-boilerplate", "info", "--verbose"]).unwrap();
        assert!(matches!(cli.command, Commands::Info { verbose: true }));
    }

    #[test]
    fn parse_run_default_mode() {
        let cli = Cli::try_parse_from(["rust-boilerplate", "run"]).unwrap();
        match cli.command {
            Commands::Run { mode } => assert_eq!(mode, "development"),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parse_api_custom_port() {
        let cli = Cli::try_parse_from(["rust-boilerplate", "api", "--port", "8080"]).unwrap();
        match cli.command {
            Commands::Api { port } => assert_eq!(port, 8080),
            _ => panic!("expected Api"),
        }
    }
}
