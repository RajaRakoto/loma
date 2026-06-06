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
        let embeddedEnv = include_str!("../json/loma_env_defaults.json");
        let sections: serde_json::Value = serde_json::from_str(embeddedEnv).unwrap_or(serde_json::json!([]));
        let mut defaultEnvContent = String::new();
        if let Some(arr) = sections.as_array() {
            for sec in arr {
                if let Some(title) = sec["section"].as_str() {
                    let lines: Vec<&str> = title.split('\n').collect();
                    if lines.len() == 1 {
                        defaultEnvContent.push_str(&format!("# ── {} ───────────────────────────────\n", title));
                    } else {
                        defaultEnvContent.push_str("# ── Configuration Scope ───────────────────────────────────────────\n");
                        for line in lines {
                            defaultEnvContent.push_str(&format!("# {}\n", line));
                        }
                    }
                }
                if let Some(vars) = sec["vars"].as_array() {
                    for var in vars {
                        let key = var["key"].as_str().unwrap_or("");
                        let value = var["value"].as_str().unwrap_or("");
                        defaultEnvContent.push_str(&format!("{}={}\n", key, value));
                    }
                }
                defaultEnvContent.push('\n');
            }
        }

        match fs::write(&lomaEnvPath, defaultEnvContent) {
            Ok(_) => display::success("Created default .loma/loma.env file."),
            Err(e) => {
                display::error(&format!("Failed to write default .loma/loma.env: {}", e));
                return Err(crate::Error::other("Failed to write loma.env"));
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
