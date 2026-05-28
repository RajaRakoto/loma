use crate::utils::display;
use std::fs;
use std::path::{Path, PathBuf};

pub fn runInit() -> crate::Result<()> {
    display::title("Initialize ccm Configuration");

    // 1. Initialize ccm.env
    let ccmEnvPath = Path::new("ccm.env");
    if ccmEnvPath.exists() {
        display::info("ccm.env configuration file already exists.");
    } else {
        display::step("Creating ccm.env configuration file...");
        let examplePath = Path::new(".env.example");
        if examplePath.exists() {
            match fs::copy(examplePath, ccmEnvPath) {
                Ok(_) => display::success("Copied .env.example to ccm.env successfully."),
                Err(e) => {
                    display::error(&format!("Failed to copy .env.example to ccm.env: {}", e));
                    return Err(crate::Error::other("Failed to copy .env.example"));
                }
            }
        } else {
            let defaultEnvContent = r#"# ccm configuration file

# CLI
CLI_ENV=development   # development | production
CLI_DEBUG=true

# API Server
API_HOST=127.0.0.1
API_PORT=3000

# Logging
RUST_LOG=ccm=debug,tower_http=info

# CLAUDE Config
CLAUDE_CODE_AUTO_COMPACT_WINDOW=190000
"#;
            match fs::write(ccmEnvPath, defaultEnvContent) {
                Ok(_) => display::success("Created default ccm.env file."),
                Err(e) => {
                    display::error(&format!("Failed to write default ccm.env: {}", e));
                    return Err(crate::Error::other("Failed to write ccm.env"));
                }
            }
        }
    }

    // 2. Initialize ~/.claude directory
    display::step("Initializing Claude configuration directory...");
    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);
        let claudeDir = homePath.join(".claude");
        if claudeDir.exists() {
            display::info("~/.claude/ directory already exists.");
        } else {
            match fs::create_dir_all(&claudeDir) {
                Ok(_) => display::success("Created ~/.claude/ directory."),
                Err(e) => {
                    display::error(&format!("Failed to create ~/.claude/ directory: {}", e));
                    return Err(crate::Error::other("Failed to create ~/.claude/ directory"));
                }
            }
        }
    } else {
        display::error("HOME environment variable not set.");
    }

    display::divider();
    display::success("Initialization completed successfully!");

    Ok(())
}
