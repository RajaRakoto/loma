use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::path::PathBuf;
use std::process::Command;

pub fn runInstall() -> crate::Result<()> {
    display::title("Installing Claude Code");

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

        if !display::confirm("Claude Code is already present. Force reinstall?") {
            display::info("Installation cancelled.");
            return Ok(());
        }

        // Remove existing installation
        crate::commands::remove::runRemove()?;
    }

    // Check for leftover files
    let mut staleFound = false;
    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);
        for d in lomaFs::CLAUDE_CONFIG_DIRS {
            if homePath.join(d).exists() {
                display::warn(&format!("Leftover directory found: ~/.{}", d));
                staleFound = true;
            }
        }
        for d in lomaFs::CLAUDE_DATA_DIRS {
            if homePath.join(d).exists() {
                display::warn(&format!("Leftover directory found: ~/{}", d));
                staleFound = true;
            }
        }
        for f in lomaFs::CLAUDE_CONFIG_FILES {
            if homePath.join(f).exists() {
                display::warn(&format!("Leftover file found: ~/.{}", f));
                staleFound = true;
            }
        }
    }

    if staleFound {
        display::warn("Leftover files were detected.");
        if display::confirm("Remove these leftover files before installing?") {
            crate::commands::remove::removeConfigsAndData()?;
        }
    } else {
        display::success("No leftover files detected. Environment is clean.");
    }

    let installStatus = if cfg!(windows) {
        display::step("Installing Claude Code globally via npm");
        display::info("Command: npm install -g @anthropic-ai/claude-code");
        display::divider();
        Command::new("cmd")
            .args(["/C", "npm install -g @anthropic-ai/claude-code"])
            .status()?
    } else {
        // Ensure curl is available
        if !lomaFs::cmdExists("curl") {
            display::step("Installing curl");
            let status = Command::new("sudo")
                .args(["dnf", "install", "-y", "curl"])
                .status()?;
            if !status.success() {
                display::error("Unable to install curl.");
                return Err(crate::Error::other("Failed to install curl"));
            }
        }

        // Run the official Anthropic install script
        display::step("Downloading and installing Claude Code");
        display::info("Command: curl -fsSL https://claude.ai/install.sh | bash");
        display::divider();

        Command::new("sh")
            .args(["-c", "curl -fsSL https://claude.ai/install.sh | bash"])
            .status()?
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
        display::info("Try: export PATH=\"$HOME/.local/bin:$PATH\"");
        return Err(crate::Error::other("Binary not found after installation"));
    }

    Ok(())
}
