use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn runRemove(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Complete Removal of {}", assistant));

    if assistant != "claude" {
        display::info(&format!("Removal logic for '{}' is not implemented yet.", assistant));
        return Ok(());
    }

    let isInstalled = lomaFs::claudeIsInstalled();
    let assistantDir = lomaFs::getAssistantDir(assistant);
    let assistantConfigFile = lomaFs::getAssistantConfigFile(assistant);

    let hasConfigDir = assistantDir.exists();
    let hasConfigFile = assistantConfigFile.exists();

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
    println!("  • .loma/{}/  (settings, agents, skills, MCP configs)", assistant);
    println!("  • .loma/{}.json  (auth tokens, session history)", assistant);
    println!();

    if !display::confirm(&format!("Confirm complete removal of {}?", assistant)) {
        display::info("Removal cancelled by user.");
        return Ok(());
    }

    removeBinaries()?;
    removeConfigsAndData(assistant)?;

    display::divider();
    display::step("Post-removal verification");
    let mut clean = true;

    if lomaFs::claudeIsInstalled() {
        display::warn(&format!(
            "A 'claude' binary is still resolved: {}",
            lomaFs::getClaudeBinary()
        ));
        clean = false;
    } else {
        display::success("'claude' binary: not found.");
    }

    if assistantDir.exists() {
        display::warn(&format!("Directory still present: {}", assistantDir.display()));
        clean = false;
    } else {
        display::success("Assistant configuration directory: removed.");
    }

    if assistantConfigFile.exists() {
        display::warn(&format!("File still present: {}", assistantConfigFile.display()));
        clean = false;
    } else {
        display::success("Assistant configuration file: removed.");
    }

    println!();
    if clean {
        display::success(&format!("{} completely removed. System is clean.", assistant));
    } else {
        display::warn("Removal mostly successful. Some items remain (see above).");
        display::info("Re-run the remove command or delete them manually.");
    }

    Ok(())
}

fn removeBinaries() -> crate::Result<()> {
    display::step("Removing binaries");

    if lomaFs::cmdExists("npm") {
        let checkPkg = Command::new("npm")
            .args(["list", "-g", "@anthropic-ai/claude-code"])
            .output();
        let isGlobal = checkPkg
            .map(|o| {
                o.status.success()
                    && String::from_utf8_lossy(&o.stdout).contains("@anthropic-ai/claude-code")
            })
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
        for p in lomaFs::CLAUDE_BINARY_PATHS {
            let fullPath = if p.starts_with('/') {
                PathBuf::from(p)
            } else {
                homePath.join(p)
            };
            if fullPath.exists() || fullPath.is_symlink() {
                let _ = lomaFs::requireRootFor(&fullPath.to_string_lossy());
            }
        }
    }

    let remaining = lomaFs::getClaudeBinary();
    if !remaining.is_empty() {
        display::warn(&format!("Binary still found on PATH: {}", remaining));
        let _ = lomaFs::requireRootFor(&remaining);
    }

    Ok(())
}

pub fn removeConfigsAndData(assistant: &str) -> crate::Result<()> {
    display::step(&format!("Removing {} configuration files and data under .loma", assistant));

    let assistantDir = lomaFs::getAssistantDir(assistant);
    if assistantDir.exists() {
        let _ = fs::remove_dir_all(&assistantDir);
        display::success(&format!("Removed: {}", assistantDir.display()));
    } else {
        display::info(&format!("Not found: {}", assistantDir.display()));
    }

    let assistantConfigFile = lomaFs::getAssistantConfigFile(assistant);
    if assistantConfigFile.exists() {
        let _ = fs::remove_file(&assistantConfigFile);
        display::success(&format!("Removed: {}", assistantConfigFile.display()));
    } else {
        display::info(&format!("Not found: {}", assistantConfigFile.display()));
    }

    // Clean npm cache pattern for anthropic/claude-code locally if npm is present
    if lomaFs::cmdExists("npm") {
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
    }

    cleanSubdirPattern("/tmp", "claude", 2);
    display::success("Temporary files in /tmp cleaned.");

    Ok(())
}

fn cleanSubdirPattern(dir: &str, pattern: &str, maxDepth: usize) {
    let path = Path::new(dir);
    if !path.exists() {
        return;
    }
    let _ = cleanSubdirPatternRecursive(path, pattern, 1, maxDepth);
}

fn cleanSubdirPatternRecursive(
    path: &Path,
    pattern: &str,
    currentDepth: usize,
    maxDepth: usize,
) -> std::io::Result<()> {
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
                let _ =
                    cleanSubdirPatternRecursive(&entryPath, pattern, currentDepth + 1, maxDepth);
            }
        }
    }
    Ok(())
}
