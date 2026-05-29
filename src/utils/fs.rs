use chrono::Local;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

/// A robust, platform-agnostic home directory resolver.
pub fn get_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .or_else(|| {
            let drive = std::env::var_os("HOMEDRIVE");
            let path = std::env::var_os("HOMEPATH");
            match (drive, path) {
                (Some(d), Some(p)) => {
                    let mut pb = PathBuf::from(d);
                    pb.push(p);
                    Some(pb)
                }
                _ => None,
            }
        })
}

/// A native, platform-agnostic check to see if a command exists in the system PATH.
pub fn cmdExists(cmd: &str) -> bool {
    let path_var = match std::env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };

    let paths = std::env::split_paths(&path_var);

    // Extensions to check (especially on Windows)
    let extensions: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ".com", ""]
    } else {
        &[""]
    };

    for path in paths {
        for ext in extensions {
            let exe_name = format!("{}{}", cmd, ext);
            let exe_path = path.join(&exe_name);
            if exe_path.is_file() {
                return true;
            }
        }
    }

    false
}

/// A native, platform-agnostic resolver to locate the `claude` binary.
pub fn getClaudeBinary() -> String {
    // 1. Check PATH env variable
    let path_var = match std::env::var_os("PATH") {
        Some(p) => p,
        None => return String::new(),
    };
    let paths = std::env::split_paths(&path_var);
    let extensions: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ".com", ""]
    } else {
        &[""]
    };

    for path in paths {
        for ext in extensions {
            let exe_name = format!("claude{}", ext);
            let exe_path = path.join(&exe_name);
            if exe_path.is_file() {
                return exe_path.to_string_lossy().to_string();
            }
        }
    }

    // 2. Check standard home paths
    if let Some(homePath) = get_home_dir() {
        for p in CLAUDE_BINARY_PATHS {
            let fullPath = if p.starts_with('/') {
                PathBuf::from(p)
            } else {
                homePath.join(p)
            };
            if fullPath.exists() {
                return fullPath.to_string_lossy().to_string();
            }

            if cfg!(windows) {
                for ext in &[".cmd", ".exe", ".bat"] {
                    let win_path = fullPath.with_extension(ext.trim_start_matches('.'));
                    if win_path.exists() {
                        return win_path.to_string_lossy().to_string();
                    }
                }
            }
        }
    }

    String::new()
}

pub fn claudeIsInstalled() -> bool {
    !getClaudeBinary().is_empty()
}

pub fn requireRootFor(path: &str) -> crate::Result<()> {
    if cfg!(windows) {
        let p = std::path::Path::new(path);
        if p.is_dir() {
            fs::remove_dir_all(p)?;
            crate::utils::display::success(&format!("Removed directory: {}", path));
        } else if p.is_file() || p.is_symlink() {
            fs::remove_file(p)?;
            crate::utils::display::success(&format!("Removed file: {}", path));
        }
        return Ok(());
    }

    // Unix implementation
    if path.starts_with("/usr/") || path.starts_with("/etc/") {
        let euid = Command::new("id")
            .arg("-u")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<u32>()
                    .unwrap_or(999)
            })
            .unwrap_or(999);

        if euid != 0 {
            crate::utils::display::warn(&format!(
                "Removing {} requires root privileges (sudo).",
                path
            ));
            let status = Command::new("sudo").args(["rm", "-rf", path]).status()?;
            if status.success() {
                crate::utils::display::success(&format!("Removed (sudo): {}", path));
                Ok(())
            } else {
                Err(crate::Error::other(format!("Failed to remove: {path}")))
            }
        } else {
            let status = Command::new("rm").args(["-rf", path]).status()?;
            if status.success() {
                crate::utils::display::success(&format!("Removed: {}", path));
                Ok(())
            } else {
                Err(crate::Error::other(format!("Failed to remove: {path}")))
            }
        }
    } else {
        let status = Command::new("rm").args(["-rf", path]).status()?;
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
        if cfg!(windows) {
            crate::utils::display::error(
                "Please download and run the Node.js installer from nodejs.org.",
            );
        } else {
            crate::utils::display::error("  sudo dnf install nodejs");
        }
        return Err(crate::Error::other("npm not found"));
    }

    let output = Command::new("node").arg("--version").output()?;
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
    // Windows doesn't typically use shell files like .bashrc / .zshrc unless running MSYS/Cygwin/WSL.
    // Early exit on Windows to ensure safe executions.
    if cfg!(windows) {
        return Ok(());
    }

    crate::utils::display::step("Cleaning shell configuration files");
    let patterns = &[
        "claude",
        "@anthropic-ai/claude-code",
        "npm-global.*claude",
        "CLAUDE",
    ];

    let homePath = get_home_dir().ok_or_else(|| crate::Error::other("Home directory not found"))?;

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
