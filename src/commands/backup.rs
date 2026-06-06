use crate::utils::display;
use crate::utils::fs as lomaFs;
use chrono::Local;
use std::fs;

pub fn runBackup(assistant: &str) -> crate::Result<()> {
    display::title(&format!("{} Configuration Backup", assistant));

    let assistantDir = lomaFs::getAssistantDir(assistant);
    let assistantConfigFile = lomaFs::getAssistantConfigFile(assistant);

    // Check if there is something to back up
    let hasData = assistantDir.exists() || assistantConfigFile.exists();

    if !hasData {
        display::error(&format!("No {} configuration found to back up.", assistant));
        return Err(crate::Error::other("No configuration found to back up"));
    }

    display::step("Select backup type");
    println!("  1) JSON config only — settings.json & settings.local.json (no auth tokens)");
    println!("  2) Full backup      — all configuration files, data, and auth tokens");
    println!();

    let mut choice = String::new();
    loop {
        print!("\x1b[1;33m\x1b[1m  Your choice [1/2]: \x1b[0m");
        use std::io::{self, Write};
        let _ = io::stdout().flush();
        choice.clear();
        if io::stdin().read_line(&mut choice).is_err() {
            return Err(crate::Error::other("Failed to read user choice"));
        }
        let trimmed = choice.trim();
        if trimmed == "1" || trimmed == "2" {
            choice = trimmed.to_string();
            break;
        }
        display::warn("Invalid choice. Enter 1 or 2.");
    }

    // Ensure archives directory exists
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
        display::step(&format!("Backing up JSON config files from {}", display_src));
        
        let settingsFile = assistantDir.join("settings.json");
        let settingsLocalFile = assistantDir.join("settings.local.json");
        let mut relativeFiles = Vec::new();
        let dir_name_os = assistantDir.file_name()
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
        display::success(&format!("JSON config backup created: {}", archivePath.display()));
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
        display::info(&format!("Items included in backup (relative to {}):", display_root));
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
