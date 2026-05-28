use crate::utils::display;
use crate::utils::fs as ccmFs;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;

pub fn runRemove() -> crate::Result<()> {
    display::title("Complete Removal of Claude Code");

    let isInstalled = ccmFs::claudeIsInstalled();
    let hasConfigDir = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".claude").exists())
        .unwrap_or(false);
    let hasConfigFile = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".claude.json").exists())
        .unwrap_or(false);

    if !isInstalled && !hasConfigDir && !hasConfigFile {
        display::warn("Claude Code does not appear to be installed on this system.");
        if !display::confirm("Continue anyway (residual cleanup)?") {
            display::info("Removal cancelled.");
            return Ok(());
        }
    }

    println!();
    display::warn("WARNING: The following will be permanently deleted:");
    println!("  • claude binaries");
    println!("  • Anthropic DNF repository");
    println!("  • ~/.claude/  (settings, agents, skills, MCP configs)");
    println!("  • ~/.claude.json  (auth tokens, session history)");
    println!("  • ~/.local/share/claude/  (application data)");
    println!("  • ~/.cache/claude/  (caches)");
    println!("  • Entries in .bashrc / .zshrc");
    println!("  • Any related systemd services");
    println!();

    if !display::confirm("Confirm complete removal of Claude Code?") {
        display::info("Removal cancelled by user.");
        return Ok(());
    }

    removeBinaries()?;
    removeDnfRepo()?;
    removeConfigsAndData()?;
    ccmFs::cleanShellConfigs()?;
    removeServices()?;

    display::divider();
    display::step("Post-removal verification");
    let mut clean = true;

    if ccmFs::claudeIsInstalled() {
        display::warn(&format!("A 'claude' binary is still resolved: {}", ccmFs::getClaudeBinary()));
        clean = false;
    } else {
        display::success("'claude' binary: not found.");
    }

    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);
        for d in ccmFs::CLAUDE_CONFIG_DIRS {
            if homePath.join(d).exists() {
                display::warn(&format!("Directory still present: ~/.{}", d));
                clean = false;
            }
        }
        for d in ccmFs::CLAUDE_DATA_DIRS {
            if homePath.join(d).exists() {
                display::warn(&format!("Directory still present: ~/{}", d));
                clean = false;
            }
        }
        for f in ccmFs::CLAUDE_CONFIG_FILES {
            if homePath.join(f).exists() {
                display::warn(&format!("File still present: ~/.{}", f));
                clean = false;
            }
        }
    }

    println!();
    if clean {
        display::success("Claude Code completely removed. System is clean.");
    } else {
        display::warn("Removal mostly successful. Some items remain (see above).");
        display::info("Re-run the remove command or delete them manually.");
    }

    Ok(())
}

fn removeBinaries() -> crate::Result<()> {
    display::step("Removing binaries");

    if ccmFs::cmdExists("npm") {
        let checkPkg = Command::new("npm")
            .args(["list", "-g", "@anthropic-ai/claude-code"])
            .output();
        let isGlobal = checkPkg
            .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("@anthropic-ai/claude-code"))
            .unwrap_or(false);

        if isGlobal {
            display::info("Uninstalling global npm package...");
            let uninstall = Command::new("npm")
                .args(["uninstall", "-g", "@anthropic-ai/claude-code"])
                .status();
            match uninstall {
                Ok(s) if s.success() => display::success("npm package removed."),
                _ => display::warn("npm uninstall failed (may not have been installed via npm)."),
            }
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);
        for p in ccmFs::CLAUDE_BINARY_PATHS {
            let fullPath = if p.starts_with('/') {
                PathBuf::from(p)
            } else {
                homePath.join(p)
            };
            if fullPath.exists() || fullPath.is_symlink() {
                let _ = ccmFs::requireRootFor(&fullPath.to_string_lossy());
            }
        }
    }

    let remaining = ccmFs::getClaudeBinary();
    if !remaining.is_empty() {
        display::warn(&format!("Binary still found on PATH: {}", remaining));
        let _ = ccmFs::requireRootFor(&remaining);
    }

    Ok(())
}

fn removeDnfRepo() -> crate::Result<()> {
    display::step("Removing Anthropic DNF/YUM repository");
    for repo in ccmFs::CLAUDE_DNF_REPO_FILES {
        let p = Path::new(repo);
        if p.exists() {
            let euid = Command::new("id")
                .arg("-u")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().unwrap_or(999))
                .unwrap_or(999);

            let status = if euid != 0 {
                Command::new("sudo").args(["rm", "-f", repo]).status()?
            } else {
                Command::new("rm").args(["-f", repo]).status()?
            };

            if status.success() {
                display::success(&format!("Repository file removed: {}", repo));
            } else {
                display::warn(&format!("Failed to remove: {}", repo));
            }
        }
    }

    if ccmFs::cmdExists("rpm") && ccmFs::cmdExists("dnf") {
        let checkPkg = Command::new("rpm").args(["-q", "claude-code"]).output();
        if let Ok(o) = checkPkg {
            if o.status.success() {
                display::info("dnf package 'claude-code' detected — removing...");
                let removePkg = Command::new("sudo")
                    .args(["dnf", "remove", "-y", "claude-code"])
                    .status();
                match removePkg {
                    Ok(s) if s.success() => display::success("dnf package removed."),
                    _ => display::warn("dnf removal failed."),
                }
            }
        }
    }

    Ok(())
}

pub fn removeConfigsAndData() -> crate::Result<()> {
    display::step("Removing configuration files and data");

    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);

        for d in ccmFs::CLAUDE_CONFIG_DIRS {
            let path = homePath.join(d);
            if path.exists() {
                let _ = fs::remove_dir_all(&path);
                display::success(&format!("Removed: ~/{}", d));
            } else {
                display::info(&format!("Not found: ~/{}", d));
            }
        }

        for f in ccmFs::CLAUDE_CONFIG_FILES {
            let path = homePath.join(f);
            if path.exists() {
                let _ = fs::remove_file(&path);
                display::success(&format!("Removed: ~/{}", f));
            } else {
                display::info(&format!("Not found: ~/{}", f));
            }
        }

        for d in ccmFs::CLAUDE_DATA_DIRS {
            let path = homePath.join(d);
            if path.exists() {
                let _ = fs::remove_dir_all(&path);
                display::success(&format!("Removed: ~/{}", d));
            } else {
                display::info(&format!("Not found: ~/{}", d));
            }
        }
    }

    let npmCacheDir = Command::new("npm")
        .args(["config", "get", "cache"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| format!("{}/.npm", h))
                .unwrap_or_else(|_| "/tmp/.npm".to_string())
        });

    cleanSubdirPattern(&npmCacheDir, "anthropic", 3);
    cleanSubdirPattern(&npmCacheDir, "claude-code", 3);
    display::success("Anthropic npm cache entries cleaned.");

    cleanSubdirPattern("/tmp", "claude", 2);
    display::success("Temporary files in /tmp cleaned.");

    Ok(())
}

fn removeServices() -> crate::Result<()> {
    display::step("Checking for systemd services");
    let services = &["claude", "claude-code", "anthropic-claude"];

    if ccmFs::cmdExists("systemctl") {
        for svc in services {
            let check = Command::new("systemctl")
                .args(["list-units", "--all", "--full"])
                .output();
            if let Ok(o) = check {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if stdout.contains(svc) {
                    display::warn(&format!("Service found: {}", svc));
                    let _ = Command::new("sudo").args(["systemctl", "stop", svc]).status();
                    let _ = Command::new("sudo").args(["systemctl", "disable", svc]).status();
                    let _ = Command::new("sudo").args(["systemctl", "daemon-reload"]).status();
                    display::success(&format!("Service {} stopped and disabled.", svc));
                }
            }
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);
        let unitDirs = &[
            homePath.join(".config/systemd/user"),
            PathBuf::from("/etc/systemd/system"),
            PathBuf::from("/usr/lib/systemd/system"),
        ];

        for dir in unitDirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let entryPath = entry.path();
                    let name = entryPath.file_name().unwrap_or_default().to_string_lossy();
                    if name.contains("claude") {
                        let _ = ccmFs::requireRootFor(&entryPath.to_string_lossy());
                    }
                }
            }
        }
    }

    Ok(())
}

fn cleanSubdirPattern(dir: &str, pattern: &str, maxDepth: usize) {
    let path = Path::new(dir);
    if !path.exists() {
        return;
    }
    let _ = cleanSubdirPatternRecursive(path, pattern, 1, maxDepth);
}

fn cleanSubdirPatternRecursive(path: &Path, pattern: &str, currentDepth: usize, maxDepth: usize) -> std::io::Result<()> {
    if currentDepth > maxDepth {
        return Ok(());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entryPath = entry.path();
            let name = entryPath.file_name().unwrap_or_default().to_string_lossy();
            if name.contains(pattern) {
                let _ = fs::remove_dir_all(&entryPath).or_else(|_| fs::remove_file(&entryPath));
            } else if entryPath.is_dir() {
                let _ = cleanSubdirPatternRecursive(&entryPath, pattern, currentDepth + 1, maxDepth);
            }
        }
    }
    Ok(())
}
