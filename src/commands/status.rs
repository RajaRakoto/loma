use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn runStatus(assistant: &str) -> crate::Result<()> {
    display::title(&format!("{} Status", assistant));

    if assistant != "claude" {
        display::info(&format!("Status logic for '{}' is not implemented yet.", assistant));
        return Ok(());
    }

    // 1. Binary check
    let binaryPath = lomaFs::getClaudeBinary();
    if !binaryPath.is_empty() {
        display::success("Claude Code binary found.");
        display::info(&format!("Binary location: {}", binaryPath));

        // Get version
        let versionOutput = Command::new(&binaryPath).arg("--version").output();
        match versionOutput {
            Ok(output) => {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                display::info(&format!("Version: {}", ver));
            }
            Err(_) => {
                display::warn("Unable to retrieve binary version.");
            }
        }
    } else {
        display::error("Claude Code binary is not installed or not in PATH.");
    }

    display::divider();

    // 2. Directories & configurations check inside .loma
    display::step("Configuration & Data Directories");
    let assistantDir = lomaFs::getAssistantDir(assistant);
    let assistantConfigFile = lomaFs::getAssistantConfigFile(assistant);

    if assistantDir.exists() {
        let size = getDirSize(&assistantDir).unwrap_or(0);
        display::success(&format!("{}/ found (Size: {} bytes)", assistantDir.display(), size));
    } else {
        display::info(&format!("{}/ not found", assistantDir.display()));
    }

    if assistantConfigFile.exists() {
        if let Ok(meta) = fs::metadata(&assistantConfigFile) {
            display::success(&format!(
                "{} found (Size: {} bytes)",
                assistantConfigFile.display(),
                meta.len()
            ));
        }
    } else {
        display::info(&format!("{} not found", assistantConfigFile.display()));
    }

    Ok(())
}

fn getDirSize(path: &Path) -> std::io::Result<u64> {
    let mut totalSize = 0;
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entryPath = entry.path();
        if entryPath.is_file() {
            totalSize += entryPath.metadata()?.len();
        } else if entryPath.is_dir() {
            totalSize += getDirSize(&entryPath)?;
        }
    }
    Ok(totalSize)
}
