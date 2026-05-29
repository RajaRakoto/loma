use crate::utils::display;
use crate::utils::fs as lomaFs;
use chrono::Local;
use std::path::Path;
use std::process::Command;

pub fn runBackup() -> crate::Result<()> {
    display::title("Claude Code Backup");

    let homePath = lomaFs::get_home_dir()
        .ok_or_else(|| crate::Error::other("HOME environment variable not set"))?;

    let settingsFile = homePath.join(".claude/settings.json");
    let settingsLocalFile = homePath.join(".claude/settings.local.json");

    // Check if there is something to back up
    let mut hasData = false;
    for d in lomaFs::CLAUDE_CONFIG_DIRS {
        if homePath.join(d).exists() {
            hasData = true;
        }
    }
    for f in lomaFs::CLAUDE_CONFIG_FILES {
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
        let mut relativeFiles = Vec::new();
        if settingsFile.exists() {
            relativeFiles.push("settings.json".to_string());
        }
        if settingsLocalFile.exists() {
            relativeFiles.push("settings.local.json".to_string());
        }

        if relativeFiles.is_empty() {
            display::error("No settings.json found in ~/.claude");
            return Err(crate::Error::other("No settings.json found"));
        }

        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archiveName)
            .arg("-C")
            .arg(homePath.join(".claude"))
            .args(&relativeFiles)
            .status()?;

        if status.success() {
            display::success(&format!("JSON config backup created: {}", archiveName));
        } else {
            display::error("Failed to create JSON config backup.");
            return Err(crate::Error::other("tar command failed"));
        }
    } else {
        display::step("Full backup");
        let mut relativeTarArgs = Vec::new();
        for d in lomaFs::CLAUDE_CONFIG_DIRS {
            let path = homePath.join(d);
            if path.exists() {
                relativeTarArgs.push(d.to_string());
            }
        }
        for f in lomaFs::CLAUDE_CONFIG_FILES {
            let path = homePath.join(f);
            if path.exists() {
                relativeTarArgs.push(f.to_string());
            }
        }
        for d in lomaFs::CLAUDE_DATA_DIRS {
            let path = homePath.join(d);
            if path.exists() {
                relativeTarArgs.push(d.to_string());
            }
        }

        if relativeTarArgs.is_empty() {
            display::error("No Claude Code files found to back up.");
            return Err(crate::Error::other("No files found to back up"));
        }

        display::info("Items included in backup (relative to HOME):");
        for item in &relativeTarArgs {
            println!("    {}", item);
        }
        println!();

        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archiveName)
            .arg("-C")
            .arg(&homePath)
            .args(&relativeTarArgs)
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
