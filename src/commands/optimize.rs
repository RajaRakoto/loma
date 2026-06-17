use crate::utils::display;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde_json::{json, Value};
use inquire::{MultiSelect, Select};

struct EnvMapping {
    envKey: &'static str,
    jsonPath: &'static str,
    valType: &'static str,
}

const ENV_MAPPINGS: &[EnvMapping] = &[
    EnvMapping { envKey: "LOMA_MODEL", jsonPath: "model", valType: "string" },
    EnvMapping { envKey: "LOMA_EFFORT_LEVEL", jsonPath: "effortLevel", valType: "string" },
    EnvMapping { envKey: "LOMA_CLEANUP_PERIOD_DAYS", jsonPath: "cleanupPeriodDays", valType: "integer" },
    EnvMapping { envKey: "LOMA_INCLUDE_CO_AUTHORED_BY", jsonPath: "includeCoAuthoredBy", valType: "boolean" },
    EnvMapping { envKey: "LOMA_AUTO_COMPACT", jsonPath: "auto_compact", valType: "boolean" },
    EnvMapping { envKey: "LOMA_FILE_COMPACT_THRESHOLD", jsonPath: "cache.CLAUDE_AUTOMATIC_FILE_COMPACT", valType: "string" },
    EnvMapping { envKey: "LOMA_DEFAULT_SONNET_MODEL", jsonPath: "env.ANTHROPIC_DEFAULT_SONNET_MODEL", valType: "string" },
    EnvMapping { envKey: "LOMA_DEFAULT_HAIKU_MODEL", jsonPath: "env.ANTHROPIC_DEFAULT_HAIKU_MODEL", valType: "string" },
    EnvMapping { envKey: "LOMA_DEFAULT_OPUS_MODEL", jsonPath: "env.ANTHROPIC_DEFAULT_OPUS_MODEL", valType: "string" },
    EnvMapping { envKey: "LOMA_SUBAGENT_MODEL", jsonPath: "env.CLAUDE_CODE_SUBAGENT_MODEL", valType: "string" },
    EnvMapping { envKey: "LOMA_MAX_THINKING_TOKENS", jsonPath: "env.MAX_THINKING_TOKENS", valType: "string" },
    EnvMapping { envKey: "LOMA_MAX_OUTPUT_TOKENS", jsonPath: "env.CLAUDE_CODE_MAX_OUTPUT_TOKENS", valType: "string" },
    EnvMapping { envKey: "LOMA_AUTOCOMPACT_PCT", jsonPath: "env.CLAUDE_AUTOCOMPACT_PCT_OVERRIDE", valType: "string" },
    EnvMapping { envKey: "LOMA_DISABLE_1M_CONTEXT", jsonPath: "env.CLAUDE_CODE_DISABLE_1M_CONTEXT", valType: "string" },
    EnvMapping { envKey: "LOMA_BASH_DEFAULT_TIMEOUT_MS", jsonPath: "env.BASH_DEFAULT_TIMEOUT_MS", valType: "string" },
    EnvMapping { envKey: "LOMA_BASH_MAX_TIMEOUT_MS", jsonPath: "env.BASH_MAX_TIMEOUT_MS", valType: "string" },
    EnvMapping { envKey: "LOMA_BASH_MAX_OUTPUT_LENGTH", jsonPath: "env.BASH_MAX_OUTPUT_LENGTH", valType: "string" },
    EnvMapping { envKey: "LOMA_DISABLE_NONESSENTIAL_TRAFFIC", jsonPath: "env.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", valType: "string" },
    EnvMapping { envKey: "LOMA_DISABLE_FEEDBACK_SURVEYS", jsonPath: "env.CLAUDE_CODE_DISABLE_FEEDBACK_SURVEYS", valType: "string" },
    EnvMapping { envKey: "LOMA_DISABLE_TELEMETRY", jsonPath: "env.DISABLE_TELEMETRY", valType: "string" },
    EnvMapping { envKey: "LOMA_DISABLE_ERROR_REPORTING", jsonPath: "env.DISABLE_ERROR_REPORTING", valType: "string" },
    EnvMapping { envKey: "LOMA_ANTHROPIC_BASE_URL", jsonPath: "env.ANTHROPIC_BASE_URL", valType: "string" },
    EnvMapping { envKey: "LOMA_ANTHROPIC_AUTH_TOKEN", jsonPath: "env.ANTHROPIC_AUTH_TOKEN", valType: "string" },
    EnvMapping { envKey: "LOMA_API_TIMEOUT_MS", jsonPath: "env.API_TIMEOUT_MS", valType: "string" },
];

pub fn runOptimize(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Optimize {} Configuration", assistant));

    if assistant.to_lowercase() != "claude" {
        display::warn("Optimization is currently only supported for the 'claude' assistant.");
        return Ok(());
    }

    display::divider();
    display::info("Select optimization to apply:");
    let options = vec![
        "1. Claude JSON Configuration",
        "2. Recommended Ignore Patterns (.claudeignore)",
        "3. Third-Party Optimization Tools",
    ];

    let choice = Select::new("Choose optimization:", options)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    match choice {
        "1. Claude JSON Configuration" => {
            if let Err(e) = runJsonOptimizationFlow() {
                display::error(&format!("JSON configuration optimization failed: {}", e));
            }
        }
        "2. Recommended Ignore Patterns (.claudeignore)" => {
            if let Err(e) = optimizeIgnoreFile() {
                display::error(&format!("Ignore file optimization failed: {}", e));
            }
        }
        "3. Third-Party Optimization Tools" => {
            if let Err(e) = setupThirdPartyToolsFlow() {
                display::error(&format!("Third-party tools setup failed: {}", e));
            }
        }
        _ => {}
    }

    display::success("Optimization complete.");
    Ok(())
}

fn runJsonOptimizationFlow() -> crate::Result<()> {
    display::step("Claude JSON Configuration Optimizations");
    display::info("Loading default configuration variables...");

    let embeddedEnv = include_str!("../json/loma_env_defaults.json");
    let sections: Value = serde_json::from_str(embeddedEnv).unwrap_or(json!([]));
    let Some(sectionsArray) = sections.as_array() else {
        return Err(crate::Error::other("Failed to parse embedded defaults."));
    };

    let mut options = Vec::new();
    for sec in sectionsArray {
        if let Some(title) = sec["section"].as_str() {
            let singleLineTitle = title.split('\n').next().unwrap_or(title);
            options.push(singleLineTitle.to_string());
        }
    }

    let selected = MultiSelect::new("Select categories to apply/merge to Claude settings:", options)
        .with_help_message("Space to select, Enter to confirm, Arrow keys to navigate")
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    if selected.is_empty() {
        display::info("No categories selected.");
        return Ok(());
    }

    let scopeChoice = match std::env::var("LOMA_CONFIG_SCOPE").unwrap_or_default().trim().to_lowercase().as_str() {
        "global" => "global".to_string(),
        "local" => "local".to_string(),
        _ => {
            let scopes = vec![
                "Apply all globally (.claude/settings.json)",
                "Apply all locally (.claude/settings.local.json)",
                "Choose individually per selected category"
            ];
            let choice = Select::new("Choose configuration scope:", scopes)
                .prompt()
                .map_err(|e| crate::Error::other(e.to_string()))?;
            if choice.starts_with("Apply all globally") {
                "global".to_string()
            } else if choice.starts_with("Apply all locally") {
                "local".to_string()
            } else {
                "custom".to_string()
            }
        }
    };

    for sec in sectionsArray {
        let title = sec["section"].as_str().unwrap_or("");
        let singleLineTitle = title.split('\n').next().unwrap_or(title);
        
        if selected.contains(&singleLineTitle.to_string()) {
            let targetScope = if scopeChoice == "custom" {
                let options = vec![
                    "Global (.claude/settings.json)",
                    "Local (.claude/settings.local.json)"
                ];
                let promptMsg = format!("Scope for '{}':", singleLineTitle);
                let choice = Select::new(&promptMsg, options)
                    .prompt()
                    .map_err(|e| crate::Error::other(e.to_string()))?;
                if choice.starts_with("Global") {
                    "global".to_string()
                } else {
                    "local".to_string()
                }
            } else {
                scopeChoice.clone()
            };

            let targetPath = if targetScope == "global" {
                PathBuf::from(".claude/settings.json")
            } else {
                PathBuf::from(".claude/settings.local.json")
            };

            display::step(&format!("Applying '{}' settings to {}", singleLineTitle, targetPath.display()));

            if let Some(vars) = sec["vars"].as_array() {
                for var in vars {
                    let key = var["key"].as_str().unwrap_or("");
                    let defaultValue = var["value"].as_str().unwrap_or("");
                    let activeValue = std::env::var(key).unwrap_or_else(|_| defaultValue.to_string());

                    if let Some(mapping) = ENV_MAPPINGS.iter().find(|m| m.envKey == key) {
                        applyMappingToFile(&targetPath, mapping, &activeValue)?;
                        display::success(&format!("Mapped {} = '{}'", key, activeValue));
                    }
                }
            }
        }
    }

    Ok(())
}

fn applyMappingToFile(targetPath: &PathBuf, mapping: &EnvMapping, value: &str) -> crate::Result<()> {
    if let Some(parent) = targetPath.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut config: Value = if targetPath.exists() {
        let content = fs::read_to_string(targetPath)?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let parts: Vec<&str> = mapping.jsonPath.split('.').collect();
    let mut current = &mut config;
    
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            match mapping.valType {
                "boolean" => {
                    let parsed = value.trim().to_lowercase() == "true";
                    current[part] = json!(parsed);
                }
                "integer" => {
                    let parsed = value.trim().parse::<i64>().unwrap_or(0);
                    current[part] = json!(parsed);
                }
                _ => {
                    current[part] = json!(value);
                }
            }
        } else {
            if !current[part].is_object() {
                current[part] = json!({});
            }
            current = &mut current[part];
        }
    }

    if mapping.jsonPath.starts_with("cache.") {
        config["cache"]["COMPACT_ON_DEMAND"] = json!(true);
    }

    let serialized = serde_json::to_string_pretty(&config)?;
    fs::write(targetPath, serialized)?;
    Ok(())
}

fn optimizeIgnoreFile() -> crate::Result<()> {
    let ignorePath = PathBuf::from(".claudeignore");
    let embeddedIgnore = include_str!("../json/claudeignore_defaults.json");
    let recommendedLines: Vec<String> = serde_json::from_str(embeddedIgnore).unwrap_or_default();

    if !ignorePath.exists() {
        display::step("Creating .claudeignore with recommended patterns...");
        let content = recommendedLines.join("\n") + "\n";
        fs::write(&ignorePath, content)?;
        display::success("Successfully created .claudeignore.");
        return Ok(());
    }

    let existingContent = fs::read_to_string(&ignorePath)?;
    let existingLines: Vec<String> = existingContent.lines().map(|s| s.trim().to_string()).collect();

    let mut addedPatterns = Vec::new();
    for line in recommendedLines {
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !existingLines.contains(&trimmed) {
            addedPatterns.push(trimmed);
        }
    }

    if addedPatterns.is_empty() {
        display::info(".claudeignore already contains all recommended ignore patterns.");
    } else {
        display::step("Updating .claudeignore with missing patterns...");
        let mut fileContent = existingContent;
        if !fileContent.ends_with('\n') && !fileContent.is_empty() {
            fileContent.push('\n');
        }

        for pattern in &addedPatterns {
            fileContent.push_str(pattern);
            fileContent.push('\n');
            display::success(&format!("Added pattern to .claudeignore: {}", pattern));
        }

        fs::write(&ignorePath, fileContent)?;
        display::success("Successfully updated .claudeignore.");
    }

    Ok(())
}

fn setupThirdPartyToolsFlow() -> crate::Result<()> {
    display::step("Third-Party Optimization Tools Setup");
    display::info("Tools will be installed locally in the '.claude' directory.");
    
    let options = vec![
        "RTK (Rust Token Kill)",
        "Caveman",
        "Token Optimizer (Claude Code Plugin)",
        "Code Review Graph",
        "Graphify",
    ];

    let selected = MultiSelect::new("Choose tools to install/configure locally:", options)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    // Create bin directory for local tools
    let _ = fs::create_dir_all(".claude/bin");

    let mut installed_tools = Vec::new();

    for tool in selected {
        match tool {
            "RTK (Rust Token Kill)" => {
                display::step("Installing RTK locally...");
                let status = Command::new("sh")
                    .env("RTK_INSTALL_DIR", ".claude/bin")
                    .args(["-c", "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh"])
                    .status()?;
                if status.success() {
                    display::success("RTK installed successfully in .claude/bin");
                    installed_tools.push("rtk");
                } else {
                    display::error("Failed to install RTK");
                }
            }
            "Caveman" => {
                display::step("Installing Caveman locally...");
                let status = Command::new("bash")
                    .args(["-c", "curl -fsSL https://raw.githubusercontent.com/JuliusBrussee/caveman/main/install.sh | bash"])
                    .status()?;
                if status.success() {
                    // Try to move it to .claude/bin
                    if let Ok(home) = std::env::var("HOME") {
                        let installed_path = format!("{}/.local/bin/caveman", home);
                        let _ = fs::rename(installed_path, ".claude/bin/caveman");
                    }
                    display::success("Caveman installed successfully in .claude/bin");
                    installed_tools.push("caveman");
                } else {
                    display::error("Failed to install Caveman");
                }
            }
            "Token Optimizer (Claude Code Plugin)" => {
                display::info("To install Token Optimizer, run inside Claude Code:");
                display::info("  /plugin install token-optimizer@alexgreensh-token-optimizer");
                installed_tools.push("token_optimizer");
            }
            "Code Review Graph" | "Graphify" => {
                display::step(&format!("Installing {} locally via venv...", tool));
                if !PathBuf::from(".claude/venv").exists() {
                    let _ = Command::new("python3").args(["-m", "venv", ".claude/venv"]).status();
                }
                
                let pkg = if tool == "Code Review Graph" { "code-review-graph" } else { "graphifyy" };
                let pip_path = if cfg!(windows) { ".claude\\venv\\Scripts\\pip" } else { ".claude/venv/bin/pip" };
                
                let status = Command::new(pip_path).args(["install", pkg]).status()?;
                
                if status.success() {
                    display::success(&format!("{} installed successfully in .claude/venv", tool));
                    if tool == "Code Review Graph" {
                        installed_tools.push("code_review_graph");
                    } else {
                        installed_tools.push("graphify");
                    }
                } else {
                    display::error(&format!("Failed to install {}", tool));
                }
            }
            _ => {}
        }
    }

    if !installed_tools.is_empty() {
        display::divider();
        display::step("Recommended Manual Setup Steps:");

        let tutorials_val: Value = serde_json::from_str(include_str!("../json/tutorials.json")).unwrap_or_else(|_| json!({}));
        for key in installed_tools {
            if let Some(tut) = tutorials_val.get(key) {
                if let Some(title) = tut["title"].as_str() {
                    println!("\n\x1b[35;1m✦ {} ✦\x1b[0m", title);
                }
                if let Some(steps) = tut["steps"].as_array() {
                    for (idx, step) in steps.iter().enumerate() {
                        if let Some(step_str) = step.as_str() {
                            println!("  \x1b[32m{}.\x1b[0m {}", idx + 1, step_str);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
