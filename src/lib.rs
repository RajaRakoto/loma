//! rust-boilerplate — A minimal, production-ready Rust starter template.
//!
//! # Modules
//! - [`api`]    — Axum HTTP server (routes, handlers, types)
//! - [`cli`]    — Clap CLI interface
//! - [`config`] — App configuration (env vars + .env)
//! - [`error`]  — Shared error type and Result alias

pub mod api;
pub mod cli;
pub mod config;
pub mod error;

pub use error::{Error, Result};

/// Application version (from Cargo.toml)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name (from Cargo.toml)
pub const NAME: &str = env!("CARGO_PKG_NAME");
