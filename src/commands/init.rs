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
        let defaultsJson = if assistant.to_lowercase() == "opencode" {
            include_str!("../json/loma_opencode_defaults.json")
        } else {
            include_str!("../json/loma_env_defaults.json")
        };
        let sections: serde_json::Value =
            serde_json::from_str(defaultsJson).unwrap_or(serde_json::json!([]));
        let mut defaultEnvContent = String::new();
        if let Some(arr) = sections.as_array() {
            for sec in arr {
                if let Some(title) = sec["section"].as_str() {
                    let lines: Vec<&str> = title.split('\n').collect();
                    if lines.len() == 1 {
                        defaultEnvContent
                            .push_str(&format!("# ── {} ───────────────────────────────\n", title));
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

    // 2. Initialize assistant-specific files
    let assistant_lower = assistant.to_lowercase();
    if assistant_lower == "claude" {
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
            display::success(&format!(
                "Created default settings file: {}",
                settingsPath.display()
            ));
        }

        let claudeMdPath = std::path::Path::new("CLAUDE.md");
        if !claudeMdPath.exists() {
            let defaultClaudeMd = r#"# Claude Project Context

Main context loaded by the assistant at the start of each session.
"#;
            fs::write(claudeMdPath, defaultClaudeMd)?;
            display::success("Created bootstrap CLAUDE.md");
        }
    } else if assistant_lower == "opencode" {
        display::step("Initializing project files for OpenCode...");

        let agentsMdPath = std::path::Path::new("AGENTS.md");
        if !agentsMdPath.exists() {
            let defaultAgentsMd = r#"# OpenCode Project Context

## Project Overview
This file guides OpenCode for optimal token efficiency and workflow.

## Quick Start
- **Provider**: Deepseek (`/connect deepseek` with your API key)
- **Model**: deepseek/deepseek-v4-flash
- **Mode**: Start in Plan mode for non-trivial tasks, Build mode for execution

## Best Practices
- Always start non-trivial tasks in **Plan mode** first
- Use `/compact` when context reaches ~50% to maintain a lean session
- Activate MCP servers only when needed for the current task
- One session = one focused task
- Run `loma optimize opencode` to tune configuration globally
"#;
            fs::write(agentsMdPath, defaultAgentsMd)?;
            display::success("Created AGENTS.md with OpenCode best practices.");
        } else {
            display::info("AGENTS.md already exists.");
        }

        // Initialize global config directory if it doesn't exist
        display::step("Initializing global OpenCode configuration...");
        if let Some(globalDir) = lomaFs::getAssistantGlobalDir("opencode") {
            fs::create_dir_all(&globalDir)?;
            display::success(&format!(
                "Ensured global config directory: {}",
                globalDir.display()
            ));

            let agentsDir = globalDir.join("agents");
            fs::create_dir_all(&agentsDir)?;
            display::success(&format!(
                "Ensured global agents directory: {}",
                agentsDir.display()
            ));
        }
    } else {
        display::step(&format!(
            "Initializing {} configuration directory...",
            assistant
        ));
        let assistantDir = lomaFs::getAssistantDir(assistant);
        if assistantDir.exists() {
            display::info(&format!(
                "Configuration directory '.loma/{}' already exists.",
                assistant
            ));
        } else {
            match fs::create_dir_all(&assistantDir) {
                Ok(_) => display::success(&format!("Created '.loma/{}' directory.", assistant)),
                Err(e) => {
                    display::error(&format!(
                        "Failed to create '.loma/{}' directory: {}",
                        assistant, e
                    ));
                    return Err(crate::Error::other("Failed to create assistant directory"));
                }
            }
        }
    }

    display::divider();
    display::success("Initialization completed successfully!");

    let assistant_lower = assistant.to_lowercase();
    if assistant_lower == "claude" {
        display::step("Security & Environment Setup Guide");
        display::info("To configure Anthropic credentials safely, it is recommended to define them manually in your environment (e.g. via shell config like .bashrc, .zshrc, config.fish, or standard environment exports):");
        println!("  export ANTHROPIC_BASE_URL=\"<your-base-url>\"");
        println!("  export ANTHROPIC_AUTH_TOKEN=\"<your-auth-token>\"");
        println!();
        display::info("Next Steps:");
        println!("  1. Review and modify the configuration in `.loma/loma.env` if needed.");
        println!("  2. Run `loma optimize claude` to generate optimized configurations.");
    } else if assistant_lower == "opencode" {
        display::step("OpenCode Setup Guide");
        display::info("To connect to Deepseek, run inside an OpenCode session:");
        println!("  /connect deepseek");
        println!("  (paste your Deepseek API key when prompted)");
        println!();
        display::info("Next Steps:");
        println!("  1. Review and modify the configuration in `.loma/loma.env` if needed.");
        println!("  2. Run `loma optimize opencode` to push global configuration.");
        println!("  3. Run `loma gen opencode` to generate advanced AGENTS.md guidelines.");
    }

    Ok(())
}
