use crate::utils::display;
use crate::utils::fs as lomaFs;
use chrono::Local;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn runRestore(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Restore {} Backup", assistant));

    let archivesDir = lomaFs::getArchivesDir();
    display::step(&format!("Searching for backup archives in {}...", archivesDir.display()));

    if !archivesDir.exists() {
        display::error("No archives directory found. Please create a backup first.");
        return Err(crate::Error::other("Archives directory does not exist"));
    }

    let mut archives = Vec::new();
    if let Ok(entries) = fs::read_dir(&archivesDir) {
        for entry in entries.flatten() {
            let entryPath = entry.path();
            if entryPath.is_file() {
                let name = entryPath.file_name().unwrap_or_default().to_string_lossy();
                let prefix = format!("{}-backup-", assistant);
                if name.starts_with(&prefix) && name.ends_with(".tar.gz") {
                    archives.push(entryPath);
                }
            }
        }
    }

    archives.sort();

    if archives.is_empty() {
        display::error(&format!(
            "No backup archives found in: {}",
            archivesDir.to_string_lossy()
        ));
        display::info(&format!("Create a backup first with: loma backup {}", assistant));
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

    display::warn(&format!("Restore will overwrite existing {} files.", assistant));
    if !display::confirm(&format!("Confirm restore from {}?", archiveName)) {
        display::info("Restore cancelled.");
        return Ok(());
    }

    display::step("Restoring files");

    let lomaDir = lomaFs::getLomaDir();
    let assistantDir = lomaFs::getAssistantDir(assistant);
    let assistantConfigFile = lomaFs::getAssistantConfigFile(assistant);

    if assistantDir.exists() || assistantConfigFile.exists() {
        let safetyTimestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let preBackupName = format!("{}-pre-restore-{}.tar.gz", assistant, safetyTimestamp);
        let preBackupPath = archivesDir.join(&preBackupName);
        display::info(&format!(
            "Creating safety backup before restore: {}",
            preBackupPath.display()
        ));

        let mut relativePreArgs = Vec::new();
        if assistantDir.exists() {
            relativePreArgs.push(assistant.to_string());
        }
        let configFilename = format!("{}.json", assistant);
        if assistantConfigFile.exists() {
            relativePreArgs.push(configFilename);
        }

        if !relativePreArgs.is_empty() {
            let _ = Command::new("tar")
                .arg("-czf")
                .arg(&preBackupPath)
                .arg("-C")
                .arg(&lomaDir)
                .args(&relativePreArgs)
                .status();
            display::success(&format!("Safety backup created: {}", preBackupPath.display()));
        }
    }

    // Determine layout: check if archive contains the configuration folder relative path
    let has_folder_prefix = contentsStr
        .lines()
        .any(|line| line.starts_with(assistant) || line.starts_with(&format!("./{}", assistant)));

    let extractStatus = if has_folder_prefix {
        // Full backup: extract directly into lomaDir
        Command::new("tar")
            .arg("-xzf")
            .arg(selectedArchive)
            .args(["-C", &lomaDir.to_string_lossy()])
            .status()?
    } else {
        // JSON-only backup: extract into .loma/<assistant>
        fs::create_dir_all(&assistantDir)?;
        Command::new("tar")
            .arg("-xzf")
            .arg(selectedArchive)
            .args(["-C", &assistantDir.to_string_lossy()])
            .status()?
    };

    if extractStatus.success() {
        display::success("Restore completed successfully.");
        display::info(&format!("Restart {} to apply the restored settings.", assistant));
    } else {
        display::error("Restore failed.");
        return Err(crate::Error::other("tar extraction failed"));
    }

    Ok(())
}
