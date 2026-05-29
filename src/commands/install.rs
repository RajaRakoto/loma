use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::process::Command;

pub fn runInstall(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Installing {}", assistant));

    if assistant != "claude" {
        display::info(&format!("Installation logic for '{}' is not implemented yet.", assistant));
        return Ok(());
    }

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
        crate::commands::remove::runRemove(assistant)?;
    }

    // Check for leftover files in local .loma
    let assistantDir = lomaFs::getAssistantDir(assistant);
    let assistantConfigFile = lomaFs::getAssistantConfigFile(assistant);
    let mut staleFound = false;

    if assistantDir.exists() {
        display::warn(&format!("Leftover configuration directory found: {}", assistantDir.display()));
        staleFound = true;
    }
    if assistantConfigFile.exists() {
        display::warn(&format!("Leftover configuration file found: {}", assistantConfigFile.display()));
        staleFound = true;
    }

    if staleFound {
        display::warn("Leftover files were detected.");
        if display::confirm("Remove these leftover files before installing?") {
            crate::commands::remove::removeConfigsAndData(assistant)?;
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
