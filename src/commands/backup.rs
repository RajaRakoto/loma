use crate::utils::display;
use crate::utils::fs as ccmFs;
use std::process::Command;
use std::path::{Path, PathBuf};
use chrono::Local;

pub fn runBackup() -> crate::Result<()> {
    display::title("Claude Code Backup");

    let home = std::env::var("HOME").map_err(|_| crate::Error::other("HOME environment variable not set"))?;
    let homePath = PathBuf::from(&home);

    let settingsFile = homePath.join(".claude/settings.json");
    let settingsLocalFile = homePath.join(".claude/settings.local.json");

    // Check if there is something to back up
    let mut hasData = false;
    for d in ccmFs::CLAUDE_CONFIG_DIRS {
        if homePath.join(d).exists() {
            hasData = true;
        }
    }
    for f in ccmFs::CLAUDE_CONFIG_FILES {
        if homePath.join(f).exists() {
            hasData = true;
        }
    }

    if !hasData {
        display::error("No Claude Code configuration found to back up.");
        return Err(crate::Error::other("No configuration found to back up"));
    }

    display::step("Select backup type");
    println!("  1) JSON config only — settings.json & settings.local.json (no auth tokens)");
    println!("  2) Full backup      — all Claude Code files, data, and auth tokens");
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

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let archiveName = if choice == "1" {
        format!("claude-backup-json-only-{}.tar.gz", timestamp)
    } else {
        format!("claude-backup-full-{}.tar.gz", timestamp)
    };

    if choice == "1" {
        display::step("Backing up JSON config files (settings.json only)");
        let mut filesToBackup = Vec::new();
        if settingsFile.exists() {
            filesToBackup.push(settingsFile.to_string_lossy().to_string());
        }
        if settingsLocalFile.exists() {
            filesToBackup.push(settingsLocalFile.to_string_lossy().to_string());
        }

        if filesToBackup.is_empty() {
            display::error("No settings.json found in ~/.claude");
            return Err(crate::Error::other("No settings.json found"));
        }

        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archiveName)
            .arg("--transform")
            .arg("s|.*/||")
            .args(&filesToBackup)
            .status()?;

        if status.success() {
            display::success(&format!("JSON config backup created: {}", archiveName));
        } else {
            display::error("Failed to create JSON config backup.");
            return Err(crate::Error::other("tar command failed"));
        }
    } else {
        display::step("Full backup");
        let mut tarArgs = Vec::new();
        for d in ccmFs::CLAUDE_CONFIG_DIRS {
            let path = homePath.join(d);
            if path.exists() {
                tarArgs.push(path.to_string_lossy().to_string());
            }
        }
        for f in ccmFs::CLAUDE_CONFIG_FILES {
            let path = homePath.join(f);
            if path.exists() {
                tarArgs.push(path.to_string_lossy().to_string());
            }
        }
        for d in ccmFs::CLAUDE_DATA_DIRS {
            let path = homePath.join(d);
            if path.exists() {
                tarArgs.push(path.to_string_lossy().to_string());
            }
        }

        if tarArgs.is_empty() {
            display::error("No Claude Code files found to back up.");
            return Err(crate::Error::other("No files found to back up"));
        }

        display::info("Items included in backup:");
        for item in &tarArgs {
            println!("    {}", item);
        }
        println!();

        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archiveName)
            .args(&tarArgs)
            .status()?;

        if status.success() {
            display::success(&format!("Full backup created: {}", archiveName));
        } else {
            display::error("Failed to create full backup.");
            return Err(crate::Error::other("tar command failed"));
        }
    }

    let archivePath = Path::new(&archiveName);
    if let Ok(meta) = archivePath.metadata() {
        let sizeKb = meta.len() as f64 / 1024.0;
        display::info(&format!("Archive size:     {:.2} KB", sizeKb));
    }

    if let Ok(absPath) = std::fs::canonicalize(archivePath) {
        display::info(&format!("Archive location: {}", absPath.to_string_lossy()));
    }

    Ok(())
}
