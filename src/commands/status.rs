use crate::utils::display;
use crate::utils::fs as ccmFs;
use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};

pub fn runStatus() -> crate::Result<()> {
    display::title("Claude Code Status");

    // 1. Binary check
    let binaryPath = ccmFs::getClaudeBinary();
    if !binaryPath.is_empty() {
        display::success("Claude Code binary found.");
        display::info(&format!("Binary location: {}", binaryPath));

        // Get version
        let versionOutput = Command::new(&binaryPath)
            .arg("--version")
            .output();
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

    // 2. Directories & configurations check
    display::step("Configuration & Data Directories");
    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);

        // check ~/.claude
        let claudeDir = homePath.join(".claude");
        if claudeDir.exists() {
            let size = getDirSize(&claudeDir).unwrap_or(0);
            display::success(&format!("~/.claude/ found (Size: {} bytes)", size));
        } else {
            display::info("~/.claude/ not found");
        }

        // check ~/.claude.json
        let claudeJson = homePath.join(".claude.json");
        if claudeJson.exists() {
            if let Ok(meta) = fs::metadata(&claudeJson) {
                display::success(&format!("~/.claude.json found (Size: {} bytes)", meta.len()));
            }
        } else {
            display::info("~/.claude.json not found (no active auth session)");
        }

        // check data/cache dirs
        for d in ccmFs::CLAUDE_DATA_DIRS {
            let path = homePath.join(d);
            if path.exists() {
                let size = getDirSize(&path).unwrap_or(0);
                display::success(&format!("~/{} found (Size: {} bytes)", d, size));
            } else {
                display::info(&format!("~/{} not found", d));
            }
        }
    }

    display::divider();

    // 3. DNF Repos
    display::step("DNF Repositories");
    let mut repoFound = false;
    for repo in ccmFs::CLAUDE_DNF_REPO_FILES {
        let p = Path::new(repo);
        if p.exists() {
            display::success(&format!("Repo file exists: {}", repo));
            repoFound = true;
        }
    }
    if !repoFound {
        display::info("No Claude DNF repository configuration files found.");
    }

    // Check DNF rpm package
    if ccmFs::cmdExists("rpm") {
        let rpmCheck = Command::new("rpm")
            .args(["-q", "claude-code"])
            .output();
        if let Ok(o) = rpmCheck {
            if o.status.success() {
                display::success(&format!("dnf package 'claude-code' is installed: {}", String::from_utf8_lossy(&o.stdout).trim()));
            } else {
                display::info("dnf package 'claude-code' is not installed.");
            }
        }
    }

    display::divider();

    // 4. Systemd services
    display::step("Systemd Services");
    if ccmFs::cmdExists("systemctl") {
        let services = &["claude", "claude-code", "anthropic-claude"];
        let mut serviceFound = false;
        for svc in services {
            let check = Command::new("systemctl")
                .args(["is-active", svc])
                .output();
            if let Ok(o) = check {
                let status = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let unitExists = Command::new("systemctl")
                    .args(["list-units", "--all", "--full"])
                    .arg(svc)
                    .output();
                if let Ok(uo) = unitExists {
                    let uStr = String::from_utf8_lossy(&uo.stdout);
                    if uStr.contains(svc) {
                        display::success(&format!("Service '{}' exists. Active status: {}", svc, status));
                        serviceFound = true;
                    }
                }
            }
        }
        if !serviceFound {
            display::info("No Claude-related systemd services are registered.");
        }
    } else {
        display::info("systemctl not available.");
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
