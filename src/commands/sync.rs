use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RegistryEntry {
    pub target: String,
    pub source: String,
    pub hash: String,
    pub r#type: String,
    pub date: String,
    pub strategy: String,
}

pub fn calculate_hash(content: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in content.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

pub fn runSync(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Sync & Repair for {}", assistant));

    match assistant.to_lowercase().as_str() {
        "claude" => runClaudeSync(),
        "opencode" => runOpenCodeSync(),
        _ => {
            display::warn("Sync command is only supported for 'claude' and 'opencode' assistants.");
            Ok(())
        }
    }
}

fn runClaudeSync() -> crate::Result<()> {
    let assistant = "claude";
    let assistantDir = lomaFs::getAssistantDir(assistant);
    if !assistantDir.exists() {
        display::error("Native Claude directory (.claude/) does not exist. Run 'loma init claude' first.");
        return Err(crate::Error::other("Missing Native Claude directory"));
    }

    display::step("Loading Registry...");
    let registry_path = PathBuf::from(".loma/registry/injections.json");
    let mut registry: HashMap<String, RegistryEntry> = if registry_path.exists() {
        let content = fs::read_to_string(&registry_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    display::info(&format!(
        "Registry loaded with {} tracked files.",
        registry.len()
    ));

    display::step("Analyzing .claude directory structure...");
    let subdirs = ["rules", "agents", "skills", "commands"];
    let mut existing_files = Vec::new();
    for subdir in &subdirs {
        let dir_path = assistantDir.join(subdir);
        if dir_path.exists() {
            if let Ok(entries) = fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                        existing_files.push(path);
                    }
                }
            }
        }
    }

    display::info(&format!(
        "Found {} markdown configuration files on disk.",
        existing_files.len()
    ));

    let mut missing_tracked = Vec::new();
    let mut untracked_files = Vec::new();
    let mut file_hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for (key, entry) in &registry {
        let path = Path::new(&entry.target);
        if !path.exists() {
            missing_tracked.push(key.clone());
        }
    }

    for file in &existing_files {
        let file_str = file.to_string_lossy().to_string();
        let is_tracked = registry.values().any(|e| e.target == file_str);
        if !is_tracked {
            untracked_files.push(file.clone());
        }

        if let Ok(content) = fs::read_to_string(file) {
            let hash = calculate_hash(&content);
            file_hashes.entry(hash).or_default().push(file.clone());
        }
    }

    if !missing_tracked.is_empty() {
        display::warn(&format!(
            "Detected {} missing tracked files:",
            missing_tracked.len()
        ));
        for key in &missing_tracked {
            if let Some(entry) = registry.get(key) {
                println!("  • {} -> {}", key, entry.target);
            }
        }
    }

    if !untracked_files.is_empty() {
        display::info(&format!(
            "Detected {} untracked native configurations:",
            untracked_files.len()
        ));
        for file in &untracked_files {
            println!("  • {}", file.display());
        }
    }

    let mut duplicate_found = false;
    for (hash, files) in &file_hashes {
        if files.len() > 1 {
            if !duplicate_found {
                display::warn("Detected duplicate configurations (same content):");
                duplicate_found = true;
            }
            println!("  Hash: {}", hash);
            for f in files {
                println!("    • {}", f.display());
            }
        }
    }

    if !duplicate_found {
        display::success("No duplicate configurations detected.");
    }

    display::step("Verifying reference files...");
    let claude_md = Path::new("CLAUDE.md");
    if !claude_md.exists() {
        display::warn("CLAUDE.md is missing. Regenerating standard bootstrap...");
        let bootstrap = r#"# Claude Project Context

Load rules, agents, skills and commands from `.claude/`.
"#;
        fs::write(claude_md, bootstrap)?;
        display::success("Regenerated CLAUDE.md bootstrap.");
    } else {
        display::success("CLAUDE.md is present.");
    }

    let settings_json = assistantDir.join("settings.json");
    if !settings_json.exists() {
        display::warn("settings.json is missing under .claude/.");
    } else if let Ok(content) = fs::read_to_string(&settings_json) {
        if serde_json::from_str::<serde_json::Value>(&content).is_err() {
            display::error("settings.json is not valid JSON!");
        } else {
            display::success("settings.json is valid JSON.");
        }
    }

    if !missing_tracked.is_empty() || !untracked_files.is_empty() {
        display::step("Repairing registry mapping...");

        for key in &missing_tracked {
            registry.remove(key);
            display::success(&format!(
                "Removed missing tracked entry from registry: {}",
                key
            ));
        }

        for file in &untracked_files {
            let filename = file.file_name().unwrap_or_default().to_string_lossy();
            let clean_key = filename
                .replace("_RULES.md", "")
                .replace("_AGENTS.md", "")
                .replace("_SKILLS.md", "")
                .replace("_COMMANDS.md", "")
                .replace('_', "-")
                .to_lowercase();

            let parent_dir = file
                .parent()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy();
            let entry_type = parent_dir.to_string();

            if let Ok(content) = fs::read_to_string(file) {
                let hash = calculate_hash(&content);
                let new_entry = RegistryEntry {
                    target: file.to_string_lossy().to_string(),
                    source: "manual-sync".to_string(),
                    hash,
                    r#type: entry_type,
                    date: chrono::Local::now().to_rfc3339(),
                    strategy: "sync".to_string(),
                };
                registry.insert(clean_key.clone(), new_entry);
                display::success(&format!(
                    "Registered untracked configuration: {} -> {}",
                    clean_key,
                    file.display()
                ));
            }
        }

        if let Some(parent) = registry_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(&registry)?;
        fs::write(&registry_path, serialized)?;
        display::success("Registry updated successfully!");
    } else {
        display::success("Registry is fully synchronized and healthy.");
    }

    display::divider();
    display::success("Sync completed successfully!");
    Ok(())
}

fn runOpenCodeSync() -> crate::Result<()> {
    display::step("Checking OpenCode configuration health...");

    // Check AGENTS.md
    let agents_md = Path::new("AGENTS.md");
    if !agents_md.exists() {
        display::warn("AGENTS.md is missing. Run 'loma init opencode' to create one.");
    } else {
        display::success("AGENTS.md is present.");
    }

    // Check global config
    if let Some(globalDir) = lomaFs::getAssistantGlobalDir("opencode") {
        if !globalDir.exists() {
            display::info("Global OpenCode config directory not found. Run 'loma init opencode'.");
        } else {
            display::success(&format!("Global config directory: {}", globalDir.display()));

            let config_file = globalDir.join("opencode.json");
            if config_file.exists() {
                if let Ok(content) = fs::read_to_string(&config_file) {
                    if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                        display::success("opencode.json is valid JSON.");
                    } else {
                        display::error("opencode.json is not valid JSON!");
                    }
                }
            } else {
                display::info("opencode.json not found. Run 'loma optimize opencode'.");
            }

            // Check sub-agents
            let agents_dir = globalDir.join("agents");
            if agents_dir.exists() {
                if let Ok(entries) = fs::read_dir(&agents_dir) {
                    let count = entries.flatten().count();
                    display::info(&format!("{} global sub-agent(s) configured.", count));
                }
            }
        }
    }

    // Check local .opencode/ overrides
    let localDir = lomaFs::getAssistantDir("opencode");
    if localDir.exists() {
        display::info("Local .opencode/ overrides are present.");
    } else {
        display::info("No local .opencode/ overrides (all config is global).");
    }

    display::divider();
    display::success("OpenCode sync completed!");
    Ok(())
}
