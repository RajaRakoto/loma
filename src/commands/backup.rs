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
        display::error(&format!("No {} configuration found to back up in .loma.", assistant));
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

    let lomaDir = lomaFs::getLomaDir();

    if choice == "1" {
        display::step(&format!("Backing up JSON config files from .loma/{}", assistant));
        
        let settingsFile = assistantDir.join("settings.json");
        let settingsLocalFile = assistantDir.join("settings.local.json");
        let mut relativeFiles = Vec::new();
        if settingsFile.exists() {
            relativeFiles.push(format!("{}/settings.json", assistant));
        }
        if settingsLocalFile.exists() {
            relativeFiles.push(format!("{}/settings.local.json", assistant));
        }

        if relativeFiles.is_empty() {
            display::error(&format!("No settings.json found in .loma/{}", assistant));
            return Err(crate::Error::other("No settings.json found"));
        }

        lomaFs::createZip(&lomaDir, &relativeFiles, &archivePath)?;
        display::success(&format!("JSON config backup created: {}", archivePath.display()));
    } else {
        display::step("Full backup");
        let mut relativeArgs = Vec::new();
        if assistantDir.exists() {
            relativeArgs.push(assistant.to_string());
        }
        let configFilename = format!("{}.json", assistant);
        if assistantConfigFile.exists() {
            relativeArgs.push(configFilename);
        }

        if relativeArgs.is_empty() {
            display::error(&format!("No {} files found to back up.", assistant));
            return Err(crate::Error::other("No files found to back up"));
        }

        display::info("Items included in backup (relative to .loma):");
        for item in &relativeArgs {
            println!("    {}", item);
        }
        println!();

        lomaFs::createZip(&lomaDir, &relativeArgs, &archivePath)?;
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
