//! Application configuration.
//!
//! Values are read from environment variables (or a `.env` file loaded by
//! `dotenvy` in `main`).  Sensible defaults are provided so the app works
//! out of the box without any configuration.
//!
//! # Environment variables
//! | Variable   | Default       | Description                        |
//! |------------|---------------|------------------------------------|
//! | APP_ENV    | development   | `development` or `production`      |
//! | APP_DEBUG  | true          | Enable extra debug output          |
//! | API_HOST   | 127.0.0.1     | Address the HTTP server binds to   |
//! | API_PORT   | 3000          | Port the HTTP server listens on    |

use crate::error::{Error, Result};
use std::env;

/// Top-level application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// `development` or `production`
    pub env: String,
    /// Enable extra debug output
    pub debug: bool,
    /// HTTP server host
    pub api_host: String,
    /// HTTP server port
    pub api_port: u16,
}

impl AppConfig {
    /// Build config from environment variables, falling back to defaults.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            env: env::var("APP_ENV").unwrap_or_else(|_| "development".into()),
            debug: env::var("APP_DEBUG")
                .unwrap_or_else(|_| "true".into())
                .parse()
                .unwrap_or(true),
            api_host: env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            api_port: env::var("API_PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .map_err(|_| Error::config("API_PORT must be a valid port number (1-65535)"))?,
        })
    }

    /// Returns `true` when running in production mode.
    pub fn is_production(&self) -> bool {
        self.env == "production"
    }

    /// Full API base URL (useful for logging / display).
    pub fn api_url(&self) -> String {
        format!("http://{}:{}", self.api_host, self.api_port)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            env: "development".into(),
            debug: true,
            api_host: "127.0.0.1".into(),
            api_port: 3000,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.env, "development");
        assert_eq!(cfg.api_port, 3000);
        assert!(!cfg.is_production());
        assert_eq!(cfg.api_url(), "http://127.0.0.1:3000");
    }

    #[test]
    fn production_flag_works() {
        let cfg = AppConfig {
            env: "production".into(),
            debug: false,
            api_host: "0.0.0.0".into(),
            api_port: 8080,
        };
        assert!(cfg.is_production());
    }
}
