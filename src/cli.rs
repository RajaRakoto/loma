//! CLI interface
//!
//! Three focused commands:
//! - `info`  — print application metadata
//! - `run`   — entry point for custom business logic
//! - `api`   — start the Axum HTTP server

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "loma", version, author, about = "", disable_version_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Print version information.
    #[arg(short = 'v', long = "version")]
    pub version: bool,
}

/// Available sub-commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ── General / Metadata ──

    /// Print application information.
    Info {
        /// Show detailed information (version, features, repo URL).
        #[arg(short, long)]
        verbose: bool,
    },

    // ── Setup & Initialization ──

    /// Initialize configuration files for loma.
    Init {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
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

    // ── Management & Optimization ──

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

    /// Generate guidelines/conventions for an assistant.
    Gen {
        /// The assistant to target (e.g. claude -> CLAUDE.md).
        #[arg(default_value = "claude")]
        assistant: String,
    },

    // ── Maintenance & Health ──

    /// Show current status of an AI assistant.
    Status {
        /// The assistant to target.
        #[arg(default_value = "claude")]
        assistant: String,
    },

    /// Perform diagnostic health checks on the assistant's environment.
    Health,

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

    // ── Future / Dev Usage (Hidden) ──

    /// Run the application (add your business logic here).
    #[command(hide = true)]
    Run {
        /// Execution mode, e.g. `development` or `production`.
        #[arg(short, long, default_value = "development")]
        mode: String,
    },

    /// Start the Axum HTTP API server.
    #[command(hide = true)]
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
        let cli = Cli::try_parse_from(["loma", "info", "--verbose"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Info { verbose: true })));
    }

    #[test]
    fn parse_run_default_mode() {
        let cli = Cli::try_parse_from(["loma", "run"]).unwrap();
        match cli.command {
            Some(Commands::Run { mode }) => assert_eq!(mode, "development"),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn parse_api_custom_port() {
        let cli = Cli::try_parse_from(["loma", "api", "--port", "8080"]).unwrap();
        match cli.command {
            Some(Commands::Api { port }) => assert_eq!(port, 8080),
            _ => panic!("expected Api"),
        }
    }

    #[test]
    fn parse_status() {
        let cli = Cli::try_parse_from(["loma", "status"]).unwrap();
        match cli.command {
            Some(Commands::Status { assistant }) => assert_eq!(assistant, "claude"),
            _ => panic!("expected Status"),
        }
    }

    #[test]
    fn parse_install() {
        let cli = Cli::try_parse_from(["loma", "install"]).unwrap();
        match cli.command {
            Some(Commands::Install { assistant }) => assert_eq!(assistant, "claude"),
            _ => panic!("expected Install"),
        }
    }

    #[test]
    fn parse_gen_custom_assistant() {
        let cli = Cli::try_parse_from(["loma", "gen", "copilot"]).unwrap();
        match cli.command {
            Some(Commands::Gen { assistant }) => assert_eq!(assistant, "copilot"),
            _ => panic!("expected Gen"),
        }
    }

    #[test]
    fn parse_version_long() {
        let cli = Cli::try_parse_from(["loma", "--version"]).unwrap();
        assert!(cli.version);
    }

    #[test]
    fn parse_version_short() {
        let cli = Cli::try_parse_from(["loma", "-v"]).unwrap();
        assert!(cli.version);
    }
}
