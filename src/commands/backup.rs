use crate::utils::display;
use crate::utils::fs as lomaFs;
use chrono::Local;
use std::fs;

pub fn runBackup(assistant: &str) -> crate::Result<()> {
    display::title(&format!("{} Configuration Backup", assistant));

    if assistant.to_lowercase() == "opencode" {
        return runOpenCodeBackup();
    }

    let assistantDir = lomaFs::getAssistantDir(assistant);
    let assistantConfigFile = lomaFs::getAssistantConfigFile(assistant);

    let hasData = assistantDir.exists() || assistantConfigFile.exists();

    if !hasData {
        display::error(&format!("No {} configuration found to back up.", assistant));
        return Err(crate::Error::other("No configuration found to back up"));
    }

    display::step("Select backup type");

    let choice = if assistant.to_lowercase() == "claude" {
        let options = vec![
            "JSON config only — settings.json & settings.local.json (no auth tokens)",
            "Full backup      — all configuration files and data",
        ];
        let selectChoice = inquire::Select::new("Choose backup type:", options)
            .prompt()
            .map_err(|e| crate::Error::other(e.to_string()))?;
        if selectChoice.starts_with("JSON config only") {
            "1".to_string()
        } else {
            "2".to_string()
        }
    } else {
        println!("  1) JSON config only — settings.json & settings.local.json (no auth tokens)");
        println!("  2) Full backup      — all configuration files, data, and auth tokens");
        println!();

        let mut ch = String::new();
        loop {
            print!("\x1b[1;33m\x1b[1m  Your choice [1/2]: \x1b[0m");
            use std::io::{self, Write};
            let _ = io::stdout().flush();
            ch.clear();
            if io::stdin().read_line(&mut ch).is_err() {
                return Err(crate::Error::other("Failed to read user choice"));
            }
            let trimmed = ch.trim();
            if trimmed == "1" || trimmed == "2" {
                ch = trimmed.to_string();
                break;
            }
            display::warn("Invalid choice. Enter 1 or 2.");
        }
        ch
    };

    let archivesDir = lomaFs::getArchivesDir();
    fs::create_dir_all(&archivesDir)?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let archiveName = if choice == "1" {
        format!("{}-backup-json-only-{}.zip", assistant, timestamp)
    } else {
        format!("{}-backup-full-{}.zip", assistant, timestamp)
    };
    let archivePath = archivesDir.join(&archiveName);

    let is_claude = assistant.to_lowercase() == "claude";
    let baseDir = if is_claude {
        std::path::PathBuf::from(".")
    } else {
        lomaFs::getLomaDir()
    };

    if choice == "1" {
        let display_src = if is_claude {
            format!("{}/", assistantDir.display())
        } else {
            format!(".loma/{}/", assistant)
        };
        display::step(&format!(
            "Backing up JSON config files from {}",
            display_src
        ));

        let settingsFile = assistantDir.join("settings.json");
        let settingsLocalFile = assistantDir.join("settings.local.json");
        let mut relativeFiles = Vec::new();
        let dir_name_os = assistantDir
            .file_name()
            .ok_or_else(|| crate::Error::other("Invalid assistant directory path"))?;
        let dir_name = dir_name_os.to_string_lossy();
        if settingsFile.exists() {
            relativeFiles.push(format!("{}/settings.json", dir_name));
        }
        if settingsLocalFile.exists() {
            relativeFiles.push(format!("{}/settings.local.json", dir_name));
        }

        if relativeFiles.is_empty() {
            display::error(&format!("No settings.json found in {}", display_src));
            return Err(crate::Error::other("No settings.json found"));
        }

        lomaFs::createZip(&baseDir, &relativeFiles, &archivePath)?;
        display::success(&format!(
            "JSON config backup created: {}",
            archivePath.display()
        ));
    } else {
        display::step("Full backup");
        let mut relativeArgs = Vec::new();
        if assistantDir.exists() {
            if let Some(name) = assistantDir.file_name() {
                relativeArgs.push(name.to_string_lossy().into_owned());
            }
        }
        if assistantConfigFile.exists() {
            if let Some(name) = assistantConfigFile.file_name() {
                relativeArgs.push(name.to_string_lossy().into_owned());
            }
        }

        if relativeArgs.is_empty() {
            display::error(&format!("No {} files found to back up.", assistant));
            return Err(crate::Error::other("No files found to back up"));
        }

        let display_root = if is_claude { "workspace root" } else { ".loma" };
        display::info(&format!(
            "Items included in backup (relative to {}):",
            display_root
        ));
        for item in &relativeArgs {
            println!("    {}", item);
        }
        println!();

        lomaFs::createZip(&baseDir, &relativeArgs, &archivePath)?;
        display::success(&format!("Full backup created: {}", archivePath.display()));
    }

    if let Ok(meta) = archivePath.metadata() {
        let sizeKb = meta.len() as f64 / 1024.0;
        display::info(&format!("Archive size:     {:.2} KB", sizeKb));
    }

    if let Ok(absPath) = std::fs::canonicalize(&archivePath) {
        display::info(&format!("Archive location: {}", absPath.to_string_lossy()));
    }

    Ok(())
}

fn copyDirRecursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if src.is_dir() {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let name = entry_path.file_name().unwrap_or_default();
            let dst_path = dst.join(name);
            if entry_path.is_dir() {
                fs::create_dir_all(&dst_path)?;
                let _ = copyDirRecursive(&entry_path, &dst_path);
            } else {
                let _ = fs::copy(&entry_path, &dst_path);
            }
        }
    }
    Ok(())
}

fn runOpenCodeBackup() -> crate::Result<()> {
    let archivesDir = lomaFs::getArchivesDir();
    fs::create_dir_all(&archivesDir)?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let archiveName = format!("opencode-backup-{}.zip", timestamp);
    let archivePath = archivesDir.join(&archiveName);

    display::step("Backing up OpenCode configuration...");

    let mut itemsToBackup = Vec::new();
    let tempDir = lomaFs::getLomaDir().join("tmp_opencode_backup");
    let _ = fs::remove_dir_all(&tempDir);
    fs::create_dir_all(&tempDir)?;

    // Backup AGENTS.md (project-level)
    let agentsMd = std::path::Path::new("AGENTS.md");
    if agentsMd.exists() {
        let dest = tempDir.join("AGENTS.md");
        fs::copy(agentsMd, &dest)?;
        itemsToBackup.push("AGENTS.md".to_string());
    }

    // Backup .opencode/ (project-level overrides)
    let localDir = lomaFs::getAssistantDir("opencode");
    if localDir.exists() {
        let dest = tempDir.join(localDir.file_name().unwrap_or_default());
        let _ = fs::create_dir_all(&dest);
        let _ = copyDirRecursive(&localDir, &dest);
        itemsToBackup.push(localDir.file_name().unwrap_or_default().to_string_lossy().to_string());
    }

    // Backup global config
    if let Some(globalDir) = lomaFs::getAssistantGlobalDir("opencode") {
        if globalDir.exists() {
            let dest = tempDir.join("global_config");
            let _ = copyDirRecursive(&globalDir, &dest);
            itemsToBackup.push("global_config".to_string());
        }
    }

    if itemsToBackup.is_empty() {
        display::warn("No OpenCode configuration found to back up.");
        display::info("Run 'loma init opencode' first to create configuration.");
        let _ = fs::remove_dir_all(&tempDir);
        return Ok(());
    }

    display::info("Items included in backup:");
    for item in &itemsToBackup {
        println!("    {}", item);
    }
    println!();

    let allFiles: Vec<String> = fs::read_dir(&tempDir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    lomaFs::createZip(&tempDir, &allFiles, &archivePath)?;

    let _ = fs::remove_dir_all(&tempDir);

    display::success(&format!("OpenCode backup created: {}", archivePath.display()));

    if let Ok(meta) = archivePath.metadata() {
        let sizeKb = meta.len() as f64 / 1024.0;
        display::info(&format!("Archive size:     {:.2} KB", sizeKb));
    }

    Ok(())
}
