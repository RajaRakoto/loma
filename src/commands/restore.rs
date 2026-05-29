use crate::utils::display;
use crate::utils::fs as lomaFs;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn runRestore() -> crate::Result<()> {
    display::title("Restore Claude Code Backup");

    display::step("Searching for backup archives");

    let currentDir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut archives = Vec::new();
    if let Ok(entries) = fs::read_dir(&currentDir) {
        for entry in entries.flatten() {
            let entryPath = entry.path();
            if entryPath.is_file() {
                let name = entryPath.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with("claude-backup-") && name.ends_with(".tar.gz") {
                    archives.push(entryPath);
                }
            }
        }
    }

    archives.sort();

    if archives.is_empty() {
        display::error(&format!(
            "No backup archives found in: {}",
            currentDir.to_string_lossy()
        ));
        display::info("Create a backup first with: loma backup");
        return Err(crate::Error::other("No backup archives found"));
    }

    display::step("Available archives");
    for (i, arc) in archives.iter().enumerate() {
        let name = arc.file_name().unwrap_or_default().to_string_lossy();
        let sizeStr = if let Ok(meta) = arc.metadata() {
            format!("{:.2} KB", meta.len() as f64 / 1024.0)
        } else {
            "unknown size".to_string()
        };
        let mtimeStr = if let Ok(meta) = arc.metadata() {
            if let Ok(modified) = meta.modified() {
                let datetime: chrono::DateTime<Local> = modified.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                "unknown date".to_string()
            }
        } else {
            "unknown date".to_string()
        };
        println!("  {}) {} ({}, {})", i + 1, name, sizeStr, mtimeStr);
    }
    println!();

    let mut choice = String::new();
    let selectedArchive: &Path;
    loop {
        print!(
            "\x1b[1;33m\x1b[1m  Select an archive [1-{}]: \x1b[0m",
            archives.len()
        );
        use std::io::{self, Write};
        let _ = io::stdout().flush();
        choice.clear();
        if io::stdin().read_line(&mut choice).is_err() {
            return Err(crate::Error::other("Failed to read user choice"));
        }
        let trimmed = choice.trim();
        if let Ok(idx) = trimmed.parse::<usize>() {
            if idx >= 1 && idx <= archives.len() {
                selectedArchive = &archives[idx - 1];
                break;
            }
        }
        display::warn("Invalid choice.");
    }

    let archiveName = selectedArchive
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    display::info(&format!("Selected archive: {}", archiveName));
    println!();

    // Show archive contents
    display::step("Archive contents");
    let tarContents = Command::new("tar")
        .arg("-tzf")
        .arg(selectedArchive)
        .output()?;
    let contentsStr = String::from_utf8_lossy(&tarContents.stdout);
    let lines = contentsStr.lines().take(30);
    for line in lines {
        println!("  {}", line);
    }
    println!();

    display::warn("Restore will overwrite existing Claude Code files.");
    if !display::confirm(&format!("Confirm restore from {}?", archiveName)) {
        display::info("Restore cancelled.");
        return Ok(());
    }

    display::step("Restoring files");

    let homePath = lomaFs::get_home_dir()
        .ok_or_else(|| crate::Error::other("HOME environment variable not set"))?;
    let configDir = homePath.join(".claude");
    let configFile = homePath.join(".claude.json");

    if configDir.exists() || configFile.exists() {
        let safetyTimestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let preBackupName = format!("claude-pre-restore-{}.tar.gz", safetyTimestamp);
        display::info(&format!(
            "Creating safety backup before restore: {}",
            preBackupName
        ));

        let mut relativePreArgs = Vec::new();
        for d in lomaFs::CLAUDE_CONFIG_DIRS {
            let path = homePath.join(d);
            if path.exists() {
                relativePreArgs.push(d.to_string());
            }
        }
        for f in lomaFs::CLAUDE_CONFIG_FILES {
            let path = homePath.join(f);
            if path.exists() {
                relativePreArgs.push(f.to_string());
            }
        }

        if !relativePreArgs.is_empty() {
            let _ = Command::new("tar")
                .arg("-czf")
                .arg(&preBackupName)
                .arg("-C")
                .arg(&homePath)
                .args(&relativePreArgs)
                .status();
            display::success(&format!("Safety backup created: {}", preBackupName));
        }
    }

    // Determine layout: check if archive contains the configuration folder relative path
    let has_claude_folder = contentsStr
        .lines()
        .any(|line| line.starts_with(".claude") || line.starts_with("./.claude"));

    let extractStatus = if has_claude_folder {
        // Full backup: extract directly into homePath
        Command::new("tar")
            .arg("-xzf")
            .arg(selectedArchive)
            .args(["-C", &homePath.to_string_lossy()])
            .status()?
    } else {
        // JSON-only backup: extract into ~/.claude
        let dest = homePath.join(".claude");
        fs::create_dir_all(&dest)?;
        Command::new("tar")
            .arg("-xzf")
            .arg(selectedArchive)
            .args(["-C", &dest.to_string_lossy()])
            .status()?
    };

    if extractStatus.success() {
        display::success("Restore completed successfully.");
        display::info("Restart Claude Code to apply the restored settings.");
    } else {
        display::error("Restore failed.");
        return Err(crate::Error::other("tar extraction failed"));
    }

    Ok(())
}
