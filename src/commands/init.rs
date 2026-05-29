use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::fs;

pub fn runInit(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Initialize loma Configuration for {}", assistant));

    // Ensure the .loma directory exists
    let lomaDir = lomaFs::getLomaDir();
    if !lomaDir.exists() {
        display::step("Creating .loma directory...");
        fs::create_dir_all(&lomaDir)?;
        display::success("Created .loma directory.");
    }

    // 1. Initialize loma.env in .loma/loma.env
    let lomaEnvPath = lomaDir.join("loma.env");
    if lomaEnvPath.exists() {
        display::info(".loma/loma.env configuration file already exists.");
    } else {
        display::step("Creating .loma/loma.env configuration file...");
        let examplePath = std::path::Path::new(".env.example");
        if examplePath.exists() {
            match fs::copy(examplePath, &lomaEnvPath) {
                Ok(_) => display::success("Copied .env.example to .loma/loma.env successfully."),
                Err(e) => {
                    display::error(&format!("Failed to copy .env.example to .loma/loma.env: {}", e));
                    return Err(crate::Error::other("Failed to copy .env.example"));
                }
            }
        } else {
            let defaultEnvContent = r#"# loma configuration file

# CLI
CLI_ENV=development   # development | production
CLI_DEBUG=true

# API Server
API_HOST=127.0.0.1
API_PORT=3000

# Logging
RUST_LOG=loma=debug,tower_http=info

# CLAUDE Config
CLAUDE_CODE_AUTO_COMPACT_WINDOW=190000
"#;
            match fs::write(&lomaEnvPath, defaultEnvContent) {
                Ok(_) => display::success("Created default .loma/loma.env file."),
                Err(e) => {
                    display::error(&format!("Failed to write default .loma/loma.env: {}", e));
                    return Err(crate::Error::other("Failed to write loma.env"));
                }
            }
        }
    }

    // 2. Initialize assistant configuration directory
    if assistant.to_lowercase() == "claude" {
        display::step("Initializing native Claude architecture...");
        let assistantDir = lomaFs::getAssistantDir(assistant);
        
        let subdirs = ["rules", "agents", "skills", "commands"];
        for subdir in &subdirs {
            let path = assistantDir.join(subdir);
            if !path.exists() {
                fs::create_dir_all(&path)?;
                display::success(&format!("Created native subdirectory: {}", path.display()));
            }
        }

        let settingsPath = assistantDir.join("settings.json");
        if !settingsPath.exists() {
            let defaultSettings = r#"{
  "watchPatterns": []
}
"#;
            fs::write(&settingsPath, defaultSettings)?;
            display::success(&format!("Created default settings file: {}", settingsPath.display()));
        }

        let claudeMdPath = std::path::Path::new("CLAUDE.md");
        if !claudeMdPath.exists() {
            let defaultClaudeMd = r#"# Claude Project Context

Load rules, agents, skills and commands from `.claude/`.
"#;
            fs::write(claudeMdPath, defaultClaudeMd)?;
            display::success("Created bootstrap CLAUDE.md");
        }
    } else {
        display::step(&format!("Initializing {} configuration directory...", assistant));
        let assistantDir = lomaFs::getAssistantDir(assistant);
        if assistantDir.exists() {
            display::info(&format!("Configuration directory '.loma/{}' already exists.", assistant));
        } else {
            match fs::create_dir_all(&assistantDir) {
                Ok(_) => display::success(&format!("Created '.loma/{}' directory.", assistant)),
                Err(e) => {
                    display::error(&format!("Failed to create '.loma/{}' directory: {}", assistant, e));
                    return Err(crate::Error::other("Failed to create assistant directory"));
                }
            }
        }
    }

    display::divider();
    display::success("Initialization completed successfully!");

    Ok(())
}
