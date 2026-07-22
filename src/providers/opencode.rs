use crate::utils::display;
use crate::utils::fs as lomaFs;
use crate::utils::r#const::OPENCODE_DATA_DIRS;
use inquire::MultiSelect;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct OpenCodeProvider;

impl Default for OpenCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenCodeProvider {
    pub fn new() -> Self {
        Self
    }

    fn removeBinaries(&self) -> crate::Result<()> {
        display::step("Removing opencode binaries");

        if let Some(home) = lomaFs::get_home_dir() {
            for p in lomaFs::OPENCODE_BINARY_PATHS {
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

        let remaining = lomaFs::getOpenCodeBinary();
        if !remaining.is_empty() {
            display::warn(&format!("Binary still found on PATH: {}", remaining));
            let _ = lomaFs::requireRootFor(&remaining);
        }

        Ok(())
    }

    fn removeGlobalConfig(&self) -> crate::Result<()> {
        display::step("Removing global opencode configuration");

        if let Some(globalDir) = lomaFs::getAssistantGlobalDir("opencode") {
            if globalDir.exists() {
                if let Some(parent) = globalDir.parent() {
                    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
                    let backup_path = parent.join(format!("opencode.bak.{}.zip", timestamp));
                    match lomaFs::createZip(parent, &["opencode".to_string()], &backup_path) {
                        Ok(_) => display::info(&format!("Backed up global config to: {}", backup_path.display())),
                        Err(e) => display::warn(&format!("Backup failed (proceeding with removal): {}", e)),
                    }
                }
                let _ = fs::remove_dir_all(&globalDir);
                display::success(&format!("Removed: {}", globalDir.display()));
            } else {
                display::info(&format!("Not found: {}", globalDir.display()));
            }
        }

        Ok(())
    }

    fn cleanSubdirPattern(&self, dir: &str, pattern: &str, maxDepth: usize) {
        let path = std::path::Path::new(dir);
        if !path.exists() {
            return;
        }
        let _ = self.cleanSubdirPatternRecursive(path, pattern, 1, maxDepth);
    }

    fn cleanSubdirPatternRecursive(
        &self,
        path: &std::path::Path,
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

    fn getDirSize(&self, path: &std::path::Path) -> std::io::Result<u64> {
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

impl super::AssistantProvider for OpenCodeProvider {
    fn install(&self, force: bool) -> crate::Result<()> {
        display::title("Installing OpenCode");

        display::step("Checking environment");

        if lomaFs::opencodeIsInstalled() {
            let binaryPath = lomaFs::getOpenCodeBinary();
            display::warn(&format!(
                "OpenCode appears to already be installed: {}",
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

            if !force && !display::confirm("OpenCode is already present. Force reinstall?") {
                display::info("Installation cancelled.");
                return Ok(());
            }

            self.remove()?;
        }

        let installStatus = if cfg!(windows) {
            display::step("Installing OpenCode via PowerShell");
            display::info("Command: iex \"& { $(irm https://opencode.ai/install.ps1) }\"");
            display::divider();
            Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    "iex \"& { $(irm https://opencode.ai/install.ps1) }\"",
                ])
                .status()?
        } else {
            display::step("Installing OpenCode via official installer");
            display::info("Command: curl -fsSL https://opencode.ai/install | bash");
            display::divider();
            Command::new("sh")
                .args(["-c", "curl -fsSL https://opencode.ai/install | bash"])
                .status()?
        };

        display::divider();

        if installStatus.success() {
            display::success("OpenCode installed successfully!");
        } else {
            display::error("Installation failed.");
            return Err(crate::Error::other("Installation command failed"));
        }

        display::step("Post-install verification");

        let binaryPathAfter = lomaFs::getOpenCodeBinary();
        if !binaryPathAfter.is_empty() {
            let versionOutput = Command::new(&binaryPathAfter).arg("--version").output();
            let ver = versionOutput
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            display::success(&format!("OpenCode is operational — version: {}", ver));
            display::info(&format!("Binary location: {}", binaryPathAfter));
            println!();
            display::info("Run 'opencode' to get started.");
        } else {
            display::error("OpenCode binary not found after installation. Check your PATH.");
            return Err(crate::Error::other("Binary not found after installation"));
        }

        Ok(())
    }

    fn reinstall(&self) -> crate::Result<()> {
        display::title("Complete Reinstall of OpenCode");
        display::info("This will first remove OpenCode entirely, then perform a clean install.");
        println!();

        if !display::confirm("Proceed with full reinstall of OpenCode?") {
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
        display::title("Interactive Removal of OpenCode");

        let isInstalled = lomaFs::opencodeIsInstalled();
        let globalDir = lomaFs::getAssistantGlobalDir("opencode");
        let hasGlobalDir = globalDir.as_ref().map(|d| d.exists()).unwrap_or(false);

        if !isInstalled && !hasGlobalDir {
            display::warn("OpenCode does not appear to be installed on this system.");
            if !display::confirm("Continue anyway (residual cleanup)?") {
                display::info("Removal cancelled.");
                return Ok(());
            }
        }

        println!();
        display::info("Select which subjects/files you want to permanently delete:");
        let options = vec![
            "Binaries (PATH binaries)",
            "Global Configuration (~/.config/opencode/)",
            "Cache & Temp Files (opencode cache directories)",
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
            } else if item.starts_with("Global Configuration") {
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
            self.removeGlobalConfig()?;
        }

        if removeCache {
            display::step("Removing cache and temporary files");
            if let Some(home) = lomaFs::get_home_dir() {
                for p in OPENCODE_DATA_DIRS {
                    let fullPath = if p.starts_with('/') {
                        PathBuf::from(p)
                    } else {
                        home.join(p)
                    };
                    if fullPath.exists() {
                        let _ = fs::remove_dir_all(&fullPath);
                        display::success(&format!("Removed: {}", fullPath.display()));
                    }
                }
            }
            self.cleanSubdirPattern("/tmp", "opencode", 2);
            display::success("Temporary files in /tmp cleaned.");
        }

        display::divider();
        display::step("Post-removal verification");
        let mut clean = true;

        if removeBinaries {
            if lomaFs::opencodeIsInstalled() {
                display::warn(&format!(
                    "An 'opencode' binary is still resolved: {}",
                    lomaFs::getOpenCodeBinary()
                ));
                clean = false;
            } else {
                display::success("'opencode' binary: not found.");
            }
        }

        if removeConfigs {
            if let Some(ref d) = globalDir {
                if d.exists() {
                    display::warn(&format!("Directory still present: {}", d.display()));
                    clean = false;
                } else {
                    display::success("Global configuration directory: removed.");
                }
            }
        }

        println!();
        if clean {
            display::success("Targeted OpenCode removal completed cleanly.");
        } else {
            display::warn("Some targeted components could not be fully removed.");
            display::info("Re-run the remove command or delete them manually.");
        }

        Ok(())
    }

    fn update(&self) -> crate::Result<()> {
        display::title("Update OpenCode");

        if !lomaFs::opencodeIsInstalled() {
            display::warn("OpenCode is not currently installed. Running clean installation instead.");
            return self.install(false);
        }

        display::step("Updating OpenCode via official installer");

        let updateStatus = if cfg!(windows) {
            Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    "iex \"& { $(irm https://opencode.ai/install.ps1) }\"",
                ])
                .status()?
        } else {
            Command::new("sh")
                .args(["-c", "curl -fsSL https://opencode.ai/install | bash"])
                .status()?
        };

        if updateStatus.success() {
            display::success("OpenCode updated successfully!");
        } else {
            display::error("Update failed.");
            return Err(crate::Error::other("Update command failed"));
        }

        let binaryPath = lomaFs::getOpenCodeBinary();
        if !binaryPath.is_empty() {
            let versionOutput = Command::new(&binaryPath).arg("--version").output();
            if let Ok(o) = versionOutput {
                display::success(&format!(
                    "OpenCode version after update: {}",
                    String::from_utf8_lossy(&o.stdout).trim()
                ));
            }
        }

        Ok(())
    }

    fn status(&self) -> crate::Result<()> {
        display::title("OpenCode Status");

        let binaryPath = lomaFs::getOpenCodeBinary();
        if !binaryPath.is_empty() {
            display::success("OpenCode binary found.");
            display::info(&format!("Binary location: {}", binaryPath));

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
            display::error("OpenCode binary is not installed or not in PATH.");
        }

        display::divider();

        display::step("Global Configuration & Data Directories");
        if let Some(globalDir) = lomaFs::getAssistantGlobalDir("opencode") {
            if globalDir.exists() {
                let size = self.getDirSize(&globalDir).unwrap_or(0);
                display::success(&format!(
                    "{}/ found (Size: {} bytes)",
                    globalDir.display(),
                    size
                ));
            } else {
                display::info(&format!("{}/ not found", globalDir.display()));
            }
        }

        let localDir = lomaFs::getAssistantDir("opencode");
        if localDir.exists() {
            let size = self.getDirSize(&localDir).unwrap_or(0);
            display::info(&format!(
                "Local .opencode/ found (Size: {} bytes)",
                size
            ));
        }

        let agentsMd = std::path::Path::new("AGENTS.md");
        if agentsMd.exists() {
            display::success("AGENTS.md is present in the project root.");
        } else {
            display::info("AGENTS.md not found. Run 'loma init opencode' to create one.");
        }

        Ok(())
    }
}
