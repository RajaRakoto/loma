use crate::utils::display;
use crate::utils::fs as ccmFs;
use std::process::Command;

pub fn runUpdate() -> crate::Result<()> {
    display::title("Update Claude Code");

    if !ccmFs::claudeIsInstalled() {
        display::warn("Claude Code is not currently installed. Running clean installation instead.");
        return crate::commands::install::runInstall();
    }

    display::step("Checking current installation type...");
    let mut npmInstalled = false;
    if ccmFs::cmdExists("npm") {
        let checkPkg = Command::new("npm")
            .args(["list", "-g", "@anthropic-ai/claude-code"])
            .output();
        npmInstalled = checkPkg
            .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("@anthropic-ai/claude-code"))
            .unwrap_or(false);
    }

    if npmInstalled {
        display::info("Claude Code was installed via npm. Updating global npm package...");
        let updateStatus = Command::new("npm")
            .args(["install", "-g", "@anthropic-ai/claude-code"])
            .status()?;
        if updateStatus.success() {
            display::success("Claude Code updated successfully via npm!");
        } else {
            display::error("npm update failed.");
            return Err(crate::Error::other("npm update failed"));
        }
    } else {
        display::info("Running official installation script to update...");
        let updateStatus = Command::new("sh")
            .args(["-c", "curl -fsSL https://claude.ai/install.sh | bash"])
            .status()?;
        if updateStatus.success() {
            display::success("Claude Code updated successfully via official installer!");
        } else {
            display::error("Official update script failed.");
            return Err(crate::Error::other("Update script failed"));
        }
    }

    // Verify after update
    let binaryPath = ccmFs::getClaudeBinary();
    if !binaryPath.is_empty() {
        let versionOutput = Command::new(&binaryPath)
            .arg("--version")
            .output();
        if let Ok(o) = versionOutput {
            display::success(&format!("Claude Code version after update: {}", String::from_utf8_lossy(&o.stdout).trim()));
        }
    }

    Ok(())
}
