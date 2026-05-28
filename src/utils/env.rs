use std::env;

/// Get environment variable or return default value.
pub fn getEnvVar(key: &str, defaultValue: &str) -> String {
    env::var(key).unwrap_or_else(|_| defaultValue.to_string())
}