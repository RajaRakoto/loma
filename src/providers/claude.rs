//! Claude-specific installation, uninstallation, updating, and status checking.

use crate::utils::display;
use crate::utils::fs as lomaFs;
use inquire::MultiSelect;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ClaudeProvider;

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeProvider {
    pub fn new() -> Self {
        Self
    }

    fn isWsl(&self) -> bool {
        if cfg!(target_os = "linux") {
            if std::env::var("WSL_DISTRO_NAME").is_ok() || std::env::var("WSL_ENV").is_ok() {
                return true;
            }
            if let Ok(contents) = fs::read_to_string("/proc/version") {
                let contents_lower = contents.to_lowercase();
                if contents_lower.contains("microsoft") || contents_lower.contains("wsl") {
                    return true;
                }
            }
        }
        false
    }

    fn removeBinaries(&self) -> crate::Result<()> {
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
                    _ => {
                        display::warn("npm uninstall failed (may not have been installed via npm).")
                    }
                }
            }
        }

        if let Some(home) = lomaFs::get_home_dir() {
            for p in lomaFs::CLAUDE_BINARY_PATHS {
                let fullPath = if p.starts_with('/') {
                    PathBuf::from(p)
                } else {
                    home.join(p)
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

    fn removeConfigsAndData(&self) -> crate::Result<()> {
        display::step("Removing configuration files and data");

        let assistantDir = lomaFs::getAssistantDir("claude");
        if assistantDir.exists() {
            let _ = fs::remove_dir_all(&assistantDir);
            display::success(&format!("Removed: {}", assistantDir.display()));
        } else {
            display::info(&format!("Not found: {}", assistantDir.display()));
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

            self.cleanSubdirPattern(&npmCacheDir, "anthropic", 3);
            self.cleanSubdirPattern(&npmCacheDir, "claude-code", 3);
            display::success("Anthropic npm cache entries cleaned.");
        }

        self.cleanSubdirPattern("/tmp", "claude", 2);
        display::success("Temporary files in /tmp cleaned.");

        // Clean shell configuration paths on Unix
        if !cfg!(windows) {
            let _ = lomaFs::cleanShellConfigs();
        }

        Ok(())
    }

    fn cleanSubdirPattern(&self, dir: &str, pattern: &str, maxDepth: usize) {
        let path = Path::new(dir);
        if !path.exists() {
            return;
        }
        let _ = self.cleanSubdirPatternRecursive(path, pattern, 1, maxDepth);
    }

    fn cleanSubdirPatternRecursive(
        &self,
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
                    let _ = self.cleanSubdirPatternRecursive(
                        &entryPath,
                        pattern,
                        currentDepth + 1,
                        maxDepth,
                    );
                }
            }
        }
        Ok(())
    }

    fn getDirSize(&self, path: &Path) -> std::io::Result<u64> {
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
                totalSize += self.getDirSize(&entryPath)?;
            }
        }
        Ok(totalSize)
    }
}

impl super::AssistantProvider for ClaudeProvider {
    fn install(&self, force: bool) -> crate::Result<()> {
        display::title("Installing Claude");

        display::step("Checking environment");

        if lomaFs::claudeIsInstalled() {
            let binaryPath = lomaFs::getClaudeBinary();
            display::warn(&format!(
                "Claude Code appears to already be installed: {}",
                binaryPath
            ));

            let versionOutput = Command::new(&binaryPath).arg("--version").output();
            if let Ok(o) = versionOutput {
                display::warn(&format!(
                    "Version: {}",
                    String::from_utf8_lossy(&o.stdout).trim()
                ));
            } else {
                display::warn("Version: unknown");
            }

            if !force && !display::confirm("Claude Code is already present. Force reinstall?") {
                display::info("Installation cancelled.");
                return Ok(());
            }

            // Remove existing installation
            self.remove()?;
        }

        let assistantDir = lomaFs::getAssistantDir("claude");
        let mut staleFound = false;

        if assistantDir.exists() {
            display::warn(&format!(
                "Leftover configuration directory found: {}",
                assistantDir.display()
            ));
            staleFound = true;
        }

        if staleFound {
            display::warn("Leftover files were detected.");
            if display::confirm("Remove these leftover files before installing?") {
                self.removeConfigsAndData()?;
            }
        } else {
            display::success("No leftover files detected. Environment is clean.");
        }

        // Perform dynamic platform-based installation
        let installStatus = if cfg!(windows) {
            display::step("Installing Claude Code via official Windows PowerShell installer");
            display::info("Command: powershell -NoProfile -ExecutionPolicy Bypass -Command \"irm https://claude.ai/install.ps1 | iex\"");
            display::divider();
            Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    "irm https://claude.ai/install.ps1 | iex",
                ])
                .status()?
        } else {
            // Linux / macOS / WSL
            let isWslEnv = self.isWsl();
            if isWslEnv {
                display::info("WSL environment detected.");
            } else if cfg!(target_os = "macos") {
                display::info("macOS environment detected.");
            } else {
                display::info("Linux environment detected.");
            }

            if lomaFs::cmdExists("curl") {
                display::step("Downloading and installing Claude Code via native installer script");
                display::info("Command: curl -fsSL https://claude.ai/install.sh | bash");
                display::divider();
                Command::new("sh")
                    .args(["-c", "curl -fsSL https://claude.ai/install.sh | bash"])
                    .status()?
            } else if lomaFs::cmdExists("npm") && lomaFs::requireNpm().is_ok() {
                display::step("curl is missing. Installing Claude Code globally via npm");
                display::info("Command: npm install -g @anthropic-ai/claude-code");
                display::divider();
                Command::new("npm")
                    .args(["install", "-g", "@anthropic-ai/claude-code"])
                    .status()?
            } else {
                display::error("Neither curl nor Node.js/npm is available.");
                display::info("Please install curl to run the recommended native installer:");
                display::info("  sudo dnf install curl  (Fedora/RHEL)");
                display::info("  sudo apt-get install curl  (Debian/Ubuntu)");
                return Err(crate::Error::other(
                    "Missing dependencies (curl or npm/nodejs)",
                ));
            }
        };

        display::divider();

        if installStatus.success() {
            display::success("Claude Code installed successfully!");
        } else {
            display::error("Installation failed.");
            return Err(crate::Error::other("Installation command failed"));
        }

        // Post-install check
        display::step("Post-install verification");

        let binaryPathAfter = lomaFs::getClaudeBinary();
        if !binaryPathAfter.is_empty() {
            let versionOutput = Command::new(&binaryPathAfter).arg("--version").output();
            let ver = versionOutput
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            display::success(&format!("Claude Code is operational — version: {}", ver));
            display::info(&format!("Binary location: {}", binaryPathAfter));
            println!();
            display::info("Run 'claude' to get started.");
        } else {
            display::error("Claude Code binary not found after installation. Check your PATH.");
            display::info("Try adding the installation directory to your PATH (e.g. export PATH=\"$HOME/.local/bin:$PATH\")");
            return Err(crate::Error::other("Binary not found after installation"));
        }

        Ok(())
    }

    fn reinstall(&self) -> crate::Result<()> {
        display::title("Complete Reinstall of Claude");
        display::info("This will first remove Claude entirely, then perform a clean install.");
        println!();

        if !display::confirm("Proceed with full reinstall of Claude?") {
            display::info("Reinstall cancelled.");
            return Ok(());
        }

        self.remove()?;
        println!();
        display::info("Waiting 2 seconds before reinstalling...");
        std::thread::sleep(std::time::Duration::from_secs(2));

        self.install(true)?;

        Ok(())
    }

    fn remove(&self) -> crate::Result<()> {
        display::title("Interactive Removal of Claude");

        let isInstalled = lomaFs::claudeIsInstalled();
        let assistantDir = lomaFs::getAssistantDir("claude");

        let hasConfigDir = assistantDir.exists();

        if !isInstalled && !hasConfigDir {
            display::warn("Claude Code does not appear to be installed on this system.");
            if !display::confirm("Continue anyway (residual cleanup)?") {
                display::info("Removal cancelled.");
                return Ok(());
            }
        }

        println!();
        display::info("Select which subjects/files you want to permanently delete:");
        let options = vec![
            "Binaries (global npm package & PATH binaries)",
            "Configurations (native .claude/ directory and settings)",
            "Cache & Temp Files (npm cache & temporary directories)",
        ];

        let selected = MultiSelect::new("Select subjects to remove:", options)
            .with_help_message("Space to select, Enter to confirm, Arrow keys to navigate")
            .prompt()
            .map_err(|e| crate::Error::other(e.to_string()))?;

        if selected.is_empty() {
            display::info("No items selected. Removal aborted.");
            return Ok(());
        }

        let mut removeBinaries = false;
        let mut removeConfigs = false;
        let mut removeCache = false;

        for item in &selected {
            if item.starts_with("Binaries") {
                removeBinaries = true;
            } else if item.starts_with("Configurations") {
                removeConfigs = true;
            } else if item.starts_with("Cache") {
                removeCache = true;
            }
        }

        println!();
        display::warn("WARNING: The selected components will be permanently deleted.");
        if !display::confirm("Confirm removal of the selected components?") {
            display::info("Removal cancelled by user.");
            return Ok(());
        }

        if removeBinaries {
            self.removeBinaries()?;
        }

        if removeConfigs {
            display::step("Removing configuration directory");
            if assistantDir.exists() {
                let _ = fs::remove_dir_all(&assistantDir);
                display::success(&format!("Removed: {}", assistantDir.display()));
            } else {
                display::info(&format!("Not found: {}", assistantDir.display()));
            }
        }

        if removeCache {
            display::step("Removing cache and temporary files");
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

                self.cleanSubdirPattern(&npmCacheDir, "anthropic", 3);
                self.cleanSubdirPattern(&npmCacheDir, "claude-code", 3);
                display::success("Anthropic npm cache entries cleaned.");
            }

            self.cleanSubdirPattern("/tmp", "claude", 2);
            display::success("Temporary files in /tmp cleaned.");

            if !cfg!(windows) {
                let _ = lomaFs::cleanShellConfigs();
            }
        }

        display::divider();
        display::step("Post-removal verification");
        let mut clean = true;

        if removeBinaries {
            if lomaFs::claudeIsInstalled() {
                display::warn(&format!(
                    "A 'claude' binary is still resolved: {}",
                    lomaFs::getClaudeBinary()
                ));
                clean = false;
            } else {
                display::success("'claude' binary: not found.");
            }
        }

        if removeConfigs {
            if assistantDir.exists() {
                display::warn(&format!(
                    "Directory still present: {}",
                    assistantDir.display()
                ));
                clean = false;
            } else {
                display::success("Assistant configuration directory: removed.");
            }
        }

        println!();
        if clean {
            display::success("Targeted Claude removal completed cleanly.");
        } else {
            display::warn("Some targeted components could not be fully removed.");
            display::info("Re-run the remove command or delete them manually.");
        }

        Ok(())
    }

    fn update(&self) -> crate::Result<()> {
        display::title("Update Claude");

        if !lomaFs::claudeIsInstalled() {
            display::warn(
                "Claude Code is not currently installed. Running clean installation instead.",
            );
            return self.install(false);
        }

        display::step("Checking current installation type...");
        let mut npmInstalled = false;
        if lomaFs::cmdExists("npm") {
            let checkPkg = Command::new("npm")
                .args(["list", "-g", "@anthropic-ai/claude-code"])
                .output();
            npmInstalled = checkPkg
                .map(|o| {
                    o.status.success()
                        && String::from_utf8_lossy(&o.stdout).contains("@anthropic-ai/claude-code")
                })
                .unwrap_or(false);
        }

        let updateStatus = if cfg!(windows) {
            if npmInstalled {
                display::info("Updating global npm package on Windows...");
                Command::new("cmd")
                    .args(["/C", "npm install -g @anthropic-ai/claude-code@latest"])
                    .status()?
            } else {
                display::info("Updating via official Windows installer...");
                Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-Command",
                        "irm https://claude.ai/install.ps1 | iex",
                    ])
                    .status()?
            }
        } else {
            // Linux/macOS/WSL
            if npmInstalled {
                display::info("Claude Code was installed via npm. Updating global npm package...");
                Command::new("npm")
                    .args(["install", "-g", "@anthropic-ai/claude-code@latest"])
                    .status()?
            } else if lomaFs::cmdExists("curl") {
                display::info("Running official installation script to update...");
                Command::new("sh")
                    .args(["-c", "curl -fsSL https://claude.ai/install.sh | bash"])
                    .status()?
            } else {
                display::error("Unable to update: curl is missing and not installed via npm.");
                return Err(crate::Error::other("Update failed: missing curl"));
            }
        };

        if updateStatus.success() {
            display::success("Claude Code updated successfully!");
        } else {
            display::error("Update failed.");
            return Err(crate::Error::other("Update command failed"));
        }

        // Verify after update
        let binaryPath = lomaFs::getClaudeBinary();
        if !binaryPath.is_empty() {
            let versionOutput = Command::new(&binaryPath).arg("--version").output();
            if let Ok(o) = versionOutput {
                display::success(&format!(
                    "Claude Code version after update: {}",
                    String::from_utf8_lossy(&o.stdout).trim()
                ));
            }
        }

        Ok(())
    }

    fn status(&self) -> crate::Result<()> {
        display::title("Claude Status");

        let assistantDir = lomaFs::getAssistantDir("claude");

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

        // 2. Directories & configurations check inside workspace
        display::step("Configuration & Data Directories");
        if assistantDir.exists() {
            let size = self.getDirSize(&assistantDir).unwrap_or(0);
            display::success(&format!(
                "{}/ found (Size: {} bytes)",
                assistantDir.display(),
                size
            ));
        } else {
            display::info(&format!("{}/ not found", assistantDir.display()));
        }

        Ok(())
    }
}
