//! CLI interface
//!
//! Three focused commands:
//! - `info`  — print application metadata
//! - `run`   — entry point for custom business logic
//! - `api`   — start the Axum HTTP server

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "loma",
    version,
    author,
    about = "Local LLM Optimizer & Manager Assistant"
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

    /// Install an AI assistant.
    Install {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Completely remove an AI assistant and all associated files.
    Remove {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Remove then cleanly reinstall an AI assistant.
    Reinstall {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Back up AI assistant configuration.
    Backup {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Restore a previous backup of an AI assistant.
    Restore {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Show current status of an AI assistant.
    Status {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Perform diagnostic health checks on the assistant's environment.
    Health,

    /// Update an AI assistant.
    Update {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Optimize configuration for an AI assistant.
    Optimize {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Generate systemd template or settings config.
    Gen,

    /// Initialize configuration files for loma.
    Init {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_info_verbose() {
        let cli = Cli::try_parse_from(["loma", "info", "--verbose"]).unwrap();
        assert!(matches!(cli.command, Commands::Info { verbose: true }));
    }

    #[test]
    fn parse_run_default_mode() {
        let cli = Cli::try_parse_from(["loma", "run"]).unwrap();
        match cli.command {
            Commands::Run { mode } => assert_eq!(mode, "development"),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parse_api_custom_port() {
        let cli = Cli::try_parse_from(["loma", "api", "--port", "8080"]).unwrap();
        match cli.command {
            Commands::Api { port } => assert_eq!(port, 8080),
            _ => panic!("expected Api"),
        }
    }

    #[test]
    fn parse_status() {
        let cli = Cli::try_parse_from(["loma", "status"]).unwrap();
        match cli.command {
            Commands::Status { assistant } => assert_eq!(assistant, "claude"),
            _ => panic!("expected Status"),
        }
    }

    #[test]
    fn parse_install() {
        let cli = Cli::try_parse_from(["loma", "install"]).unwrap();
        match cli.command {
            Commands::Install { assistant } => assert_eq!(assistant, "claude"),
            _ => panic!("expected Install"),
        }
    }
}
