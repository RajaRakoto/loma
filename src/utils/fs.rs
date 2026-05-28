use std::fs;
use std::path::PathBuf;
use std::process::Command;
use chrono::Local;

pub const CLAUDE_CONFIG_DIRS: &[&str] = &[".claude"];
pub const CLAUDE_CONFIG_FILES: &[&str] = &[".claude.json"];
pub const CLAUDE_DATA_DIRS: &[&str] = &[
    ".local/share/claude",
    ".cache/claude",
    ".cache/@anthropic-ai",
];
pub const CLAUDE_DNF_REPO_FILES: &[&str] = &[
    "/etc/yum.repos.d/claude-code.repo",
    "/etc/yum.repos.d/anthropic-claude.repo",
];
pub const CLAUDE_BINARY_PATHS: &[&str] = &[
    ".local/bin/claude",
    "/usr/local/bin/claude",
    "/usr/bin/claude",
    ".npm-global/bin/claude",
    ".bun/bin/claude",
];

pub fn cmdExists(cmd: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {cmd}")])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn getClaudeBinary() -> String {
    if cmdExists("claude") {
        if let Ok(output) = Command::new("which").arg("claude").output() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return path;
            }
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);
        for p in CLAUDE_BINARY_PATHS {
            let fullPath = if p.starts_with('/') {
                PathBuf::from(p)
            } else {
                homePath.join(p)
            };
            if fullPath.exists() {
                return fullPath.to_string_lossy().to_string();
            }
        }
    }

    String::new()
}

pub fn claudeIsInstalled() -> bool {
    !getClaudeBinary().is_empty()
}

pub fn requireRootFor(path: &str) -> crate::Result<()> {
    if path.starts_with("/usr/") || path.starts_with("/etc/") {
        let euid = Command::new("id")
            .arg("-u")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().unwrap_or(999))
            .unwrap_or(999);

        if euid != 0 {
            crate::utils::display::warn(&format!("Removing {} requires root privileges (sudo).", path));
            let status = Command::new("sudo")
                .args(["rm", "-rf", path])
                .status()?;
            if status.success() {
                crate::utils::display::success(&format!("Removed (sudo): {}", path));
                Ok(())
            } else {
                Err(crate::Error::other(format!("Failed to remove: {path}")))
            }
        } else {
            let status = Command::new("rm")
                .args(["-rf", path])
                .status()?;
            if status.success() {
                crate::utils::display::success(&format!("Removed: {}", path));
                Ok(())
            } else {
                Err(crate::Error::other(format!("Failed to remove: {path}")))
            }
        }
    } else {
        let status = Command::new("rm")
            .args(["-rf", path])
            .status()?;
        if status.success() {
            crate::utils::display::success(&format!("Removed: {}", path));
            Ok(())
        } else {
            Err(crate::Error::other(format!("Failed to remove: {path}")))
        }
    }
}

pub fn requireNpm() -> crate::Result<()> {
    if !cmdExists("npm") {
        crate::utils::display::error("npm is not installed. Please install Node.js >= 18 first.");
        crate::utils::display::error("  sudo dnf install nodejs");
        return Err(crate::Error::other("npm not found"));
    }

    let output = Command::new("node")
        .arg("--version")
        .output()?;
    let versionStr = String::from_utf8_lossy(&output.stdout);
    let cleanVersion = versionStr.trim().trim_start_matches('v');
    let majorVersion = cleanVersion
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    if majorVersion < 18 {
        crate::utils::display::error(&format!(
            "Node.js {} detected. Claude Code requires Node.js >= 18.",
            cleanVersion
        ));
        return Err(crate::Error::other("Node.js version too low"));
    }

    Ok(())
}

pub fn cleanShellConfigs() -> crate::Result<()> {
    crate::utils::display::step("Cleaning shell configuration files");
    let patterns = &[
        "claude",
        "@anthropic-ai/claude-code",
        "npm-global.*claude",
        "CLAUDE",
    ];

    let home = std::env::var("HOME").map_err(|_| crate::Error::other("HOME environment variable not set"))?;
    let homePath = PathBuf::from(home);

    let shellConfigs = &[
        homePath.join(".bashrc"),
        homePath.join(".zshrc"),
        homePath.join(".profile"),
        homePath.join(".bash_profile"),
        homePath.join(".config/fish/config.fish"),
    ];

    for cfg in shellConfigs {
        if !cfg.exists() {
            continue;
        }

        let content = fs::read_to_string(cfg)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut backupMade = false;
        let mut modified = false;

        for pat in patterns {
            let containsPattern = |line: &str, pat: &str| -> bool {
                let lineLower = line.to_lowercase();
                let patLower = pat.to_lowercase();
                if pat == "npm-global.*claude" {
                    if let Some(idx) = lineLower.find("npm-global") {
                        lineLower[idx..].contains("claude")
                    } else {
                        false
                    }
                } else {
                    lineLower.contains(&patLower)
                }
            };

            let mut linesToKeep = Vec::new();
            let mut removedAny = false;
            for line in &lines {
                if containsPattern(line, pat) {
                    removedAny = true;
                } else {
                    linesToKeep.push(line.clone());
                }
            }

            if removedAny {
                if !backupMade {
                    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
                    let backupPath = cfg.with_extension(format!("claude-backup.{}", timestamp));
                    fs::copy(cfg, &backupPath)?;
                    backupMade = true;
                }
                lines = linesToKeep;
                modified = true;
                crate::utils::display::success(&format!(
                    "Removed '{}' entries from {}",
                    pat,
                    cfg.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }

        if modified {
            let mut newContent = lines.join("\n");
            if !newContent.is_empty() && !newContent.ends_with('\n') {
                newContent.push('\n');
            }
            fs::write(cfg, newContent)?;
        }
    }

    Ok(())
}