use crate::utils::display;
use inquire::{MultiSelect, Select};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct EnvMapping {
    envKey: &'static str,
    jsonPath: &'static str,
    valType: &'static str,
}

const ENV_MAPPINGS: &[EnvMapping] = &[
    EnvMapping {
        envKey: "LOMA_CLAUDE_MODEL",
        jsonPath: "model",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_EFFORT_LEVEL",
        jsonPath: "effortLevel",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_CLEANUP_PERIOD_DAYS",
        jsonPath: "cleanupPeriodDays",
        valType: "integer",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_INCLUDE_CO_AUTHORED_BY",
        jsonPath: "includeCoAuthoredBy",
        valType: "boolean",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_AUTO_COMPACT",
        jsonPath: "auto_compact",
        valType: "boolean",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_FILE_COMPACT_THRESHOLD",
        jsonPath: "cache.CLAUDE_AUTOMATIC_FILE_COMPACT",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_DEFAULT_SONNET_MODEL",
        jsonPath: "env.ANTHROPIC_DEFAULT_SONNET_MODEL",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_DEFAULT_HAIKU_MODEL",
        jsonPath: "env.ANTHROPIC_DEFAULT_HAIKU_MODEL",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_DEFAULT_OPUS_MODEL",
        jsonPath: "env.ANTHROPIC_DEFAULT_OPUS_MODEL",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_SUBAGENT_MODEL",
        jsonPath: "env.CLAUDE_CODE_SUBAGENT_MODEL",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_MAX_THINKING_TOKENS",
        jsonPath: "env.MAX_THINKING_TOKENS",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_MAX_OUTPUT_TOKENS",
        jsonPath: "env.CLAUDE_CODE_MAX_OUTPUT_TOKENS",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_AUTOCOMPACT_PCT",
        jsonPath: "env.CLAUDE_AUTOCOMPACT_PCT_OVERRIDE",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_DISABLE_1M_CONTEXT",
        jsonPath: "env.CLAUDE_CODE_DISABLE_1M_CONTEXT",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_BASH_DEFAULT_TIMEOUT_MS",
        jsonPath: "env.BASH_DEFAULT_TIMEOUT_MS",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_BASH_MAX_TIMEOUT_MS",
        jsonPath: "env.BASH_MAX_TIMEOUT_MS",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_BASH_MAX_OUTPUT_LENGTH",
        jsonPath: "env.BASH_MAX_OUTPUT_LENGTH",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_DISABLE_NONESSENTIAL_TRAFFIC",
        jsonPath: "env.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_DISABLE_FEEDBACK_SURVEYS",
        jsonPath: "env.CLAUDE_CODE_DISABLE_FEEDBACK_SURVEYS",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_DISABLE_TELEMETRY",
        jsonPath: "env.DISABLE_TELEMETRY",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_DISABLE_ERROR_REPORTING",
        jsonPath: "env.DISABLE_ERROR_REPORTING",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_ANTHROPIC_BASE_URL",
        jsonPath: "env.ANTHROPIC_BASE_URL",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_ANTHROPIC_AUTH_TOKEN",
        jsonPath: "env.ANTHROPIC_AUTH_TOKEN",
        valType: "string",
    },
    EnvMapping {
        envKey: "LOMA_CLAUDE_API_TIMEOUT_MS",
        jsonPath: "env.API_TIMEOUT_MS",
        valType: "string",
    },
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

    let selected = MultiSelect::new(
        "Select categories to apply/merge to Claude settings:",
        options,
    )
    .with_help_message("Space to select, Enter to confirm, Arrow keys to navigate")
    .prompt()
    .map_err(|e| crate::Error::other(e.to_string()))?;

    if selected.is_empty() {
        display::info("No categories selected.");
        return Ok(());
    }

    let scopeChoice = match std::env::var("LOMA_CONFIG_SCOPE")
        .unwrap_or_default()
        .trim()
        .to_lowercase()
        .as_str()
    {
        "global" => "global".to_string(),
        "local" => "local".to_string(),
        _ => {
            let scopes = vec![
                "Apply all globally (.claude/settings.json)",
                "Apply all locally (.claude/settings.local.json)",
                "Choose individually per selected category",
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
                    "Local (.claude/settings.local.json)",
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

            display::step(&format!(
                "Applying '{}' settings to {}",
                singleLineTitle,
                targetPath.display()
            ));

            if let Some(vars) = sec["vars"].as_array() {
                for var in vars {
                    let key = var["key"].as_str().unwrap_or("");
                    let defaultValue = var["value"].as_str().unwrap_or("");
                    let activeValue =
                        std::env::var(key).unwrap_or_else(|_| defaultValue.to_string());

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

fn applyMappingToFile(
    targetPath: &PathBuf,
    mapping: &EnvMapping,
    value: &str,
) -> crate::Result<()> {
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
    optimizeIgnoreFileAtPath(&PathBuf::from(".claudeignore"))
}

fn optimizeIgnoreFileAtPath(ignorePath: &std::path::Path) -> crate::Result<()> {
    let embeddedIgnore = include_str!("../json/claudeignore_defaults.json");
    let recommendedLines: Vec<String> = serde_json::from_str(embeddedIgnore).unwrap_or_default();

    if !ignorePath.exists() {
        display::step(&format!(
            "Creating {} with recommended patterns...",
            ignorePath.display()
        ));
        let content = recommendedLines.join("\n") + "\n";
        fs::write(ignorePath, content)?;
        display::success(&format!("Successfully created {}.", ignorePath.display()));
        return Ok(());
    }

    let existingContent = fs::read_to_string(ignorePath)?;
    let existingLines: Vec<String> = existingContent
        .lines()
        .map(|s| s.trim().to_string())
        .collect();

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
        display::info(&format!(
            "{} already contains all recommended ignore patterns.",
            ignorePath.display()
        ));
    } else {
        display::step(&format!(
            "Updating {} with missing patterns...",
            ignorePath.display()
        ));
        let mut fileContent = existingContent;
        if !fileContent.ends_with('\n') && !fileContent.is_empty() {
            fileContent.push('\n');
        }

        for pattern in &addedPatterns {
            fileContent.push_str(pattern);
            fileContent.push('\n');
            display::success(&format!(
                "Added pattern to {}: {}",
                ignorePath.display(),
                pattern
            ));
        }

        fs::write(ignorePath, fileContent)?;
        display::success(&format!("Successfully updated {}.", ignorePath.display()));
    }

    Ok(())
}

fn setupThirdPartyToolsFlow() -> crate::Result<()> {
    display::step("Third-Party Optimization Tools Setup");

    let options = vec![
        "RTK (Rust Token Kill)",
        "Caveman (Plugin)",
        "Token Optimizer (Plugin)",
        "Code Review Graph",
        "Graphify",
    ];

    let selected = MultiSelect::new("Choose tools to install/configure:", options)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    let mut installed_tools = Vec::new();

    let home = std::env::var("HOME").unwrap_or_default();
    let local_bin = if home.is_empty() {
        "/usr/local/bin".to_string()
    } else {
        format!("{}/.local/bin", home)
    };

    for tool in selected {
        match tool {
            "RTK (Rust Token Kill)" => {
                display::step("Installing RTK globally...");
                let _ = fs::create_dir_all(&local_bin);
                let status = Command::new("sh")
                    .env("RTK_INSTALL_DIR", &local_bin)
                    .args(["-c", "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh"])
                    .status()?;
                if status.success() {
                    display::success(&format!("RTK installed successfully in {}", local_bin));
                    installed_tools.push("rtk");
                } else {
                    display::error("Failed to install RTK");
                }
            }
            "Caveman (Plugin)" => {
                display::step("Installing Caveman globally...");
                let status = Command::new("bash")
                    .args(["-c", "curl -fsSL https://raw.githubusercontent.com/JuliusBrussee/caveman/main/install.sh | bash"])
                    .status()?;
                if status.success() {
                    display::success("Caveman installed successfully globally");
                    installed_tools.push("caveman");
                } else {
                    display::error("Failed to install Caveman");
                }
            }
            "Token Optimizer (Plugin)" => {
                display::info("To install Token Optimizer, run inside Claude Code:");
                display::info("  /plugin install token-optimizer@alexgreensh-token-optimizer");
                installed_tools.push("token_optimizer");
            }
            "Code Review Graph" | "Graphify" => {
                display::step(&format!("Installing {} globally...", tool));
                let pkg = if tool == "Code Review Graph" {
                    "code-review-graph"
                } else {
                    "graphifyy"
                };

                let mut status = Command::new("pip3")
                    .args(["install", "--user", pkg])
                    .status();

                if status.is_err() || !status.as_ref().unwrap().success() {
                    status = Command::new("pip")
                        .args(["install", "--user", pkg])
                        .status();
                }

                if let Ok(ref s) = status {
                    if !s.success() {
                        let _ = Command::new("pip3")
                            .args(["install", "--user", "--break-system-packages", pkg])
                            .status();
                        status = Command::new("pip")
                            .args(["install", "--user", "--break-system-packages", pkg])
                            .status();
                    }
                }

                let success = status.map(|s| s.success()).unwrap_or(false);
                if success {
                    display::success(&format!("{} installed successfully globally", tool));
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

        let tutorials_val: Value = serde_json::from_str(include_str!("../json/tutorials.json"))
            .unwrap_or_else(|_| json!({}));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_mapping_to_file_string() {
        let test_dir = PathBuf::from("tmp/test_apply_mapping_string");
        let _ = fs::create_dir_all(&test_dir);
        let test_file = test_dir.join("settings.json");
        if test_file.exists() {
            let _ = fs::remove_file(&test_file);
        }

        let mapping = EnvMapping {
            envKey: "LOMA_CLAUDE_MODEL",
            jsonPath: "model",
            valType: "string",
        };

        applyMappingToFile(&test_file, &mapping, "claude-sonnet-4-5").unwrap();

        let content = fs::read_to_string(&test_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["model"], "claude-sonnet-4-5");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_apply_mapping_to_file_boolean_and_nested() {
        let test_dir = PathBuf::from("tmp/test_apply_mapping_bool");
        let _ = fs::create_dir_all(&test_dir);
        let test_file = test_dir.join("settings.json");
        if test_file.exists() {
            let _ = fs::remove_file(&test_file);
        }

        let mapping = EnvMapping {
            envKey: "LOMA_CLAUDE_AUTO_COMPACT",
            jsonPath: "cache.COMPACT_ON_DEMAND",
            valType: "boolean",
        };

        applyMappingToFile(&test_file, &mapping, "true").unwrap();

        let content = fs::read_to_string(&test_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["cache"]["COMPACT_ON_DEMAND"], true);

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_optimize_ignore_file_at_path() {
        let test_dir = PathBuf::from("tmp/test_optimize_ignore");
        let _ = fs::create_dir_all(&test_dir);
        let test_file = test_dir.join(".claudeignore");
        if test_file.exists() {
            let _ = fs::remove_file(&test_file);
        }

        // Test creation
        optimizeIgnoreFileAtPath(&test_file).unwrap();
        assert!(test_file.exists());
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains(".git"));

        // Test update/merge
        let custom_content = ".git\n# custom comment\n/node_modules/\n";
        fs::write(&test_file, custom_content).unwrap();
        optimizeIgnoreFileAtPath(&test_file).unwrap();
        let updated_content = fs::read_to_string(&test_file).unwrap();
        assert!(updated_content.contains("# custom comment"));
        assert!(updated_content.contains(".git"));

        let _ = fs::remove_dir_all(&test_dir);
    }
}
