use crate::utils::display;
use crate::utils::fs as ccmFs;
use std::process::Command;
use std::fs;
use std::path::PathBuf;

pub fn runHealth() -> crate::Result<()> {
    display::title("Claude Code Environment Health Check");

    let mut healthy = true;

    // 1. Check Node.js & npm
    display::step("Checking Node.js & npm...");
    if ccmFs::requireNpm().is_ok() {
        display::success("Node.js >= 18 and npm are present and healthy.");
    } else {
        display::error("Node.js or npm validation failed.");
        healthy = false;
    }

    // 2. Check curl
    display::step("Checking curl...");
    if ccmFs::cmdExists("curl") {
        display::success("curl is available.");
    } else {
        display::warn("curl is missing. Installation requires curl (will try to install it automatically).");
    }

    // 3. Check Fedora/DNF
    display::step("Checking package manager (dnf)...");
    if ccmFs::cmdExists("dnf") {
        display::success("dnf package manager is available.");
    } else {
        display::warn("dnf is not available. System-level integrations (repos, package) might fail.");
    }

    // 4. Check write permissions to HOME
    display::step("Checking write permissions...");
    if let Some(home) = std::env::var_os("HOME") {
        let homePath = PathBuf::from(home);
        let testFile = homePath.join(".claude_health_test");
        match fs::write(&testFile, "test") {
            Ok(_) => {
                let _ = fs::remove_file(&testFile);
                display::success("Home directory is writable.");
            }
            Err(e) => {
                display::error(&format!("Home directory is not writable: {}", e));
                healthy = false;
            }
        }
    } else {
        display::error("HOME environment variable not set.");
        healthy = false;
    }

    // 5. Check internet connectivity to registry.npmjs.org
    display::step("Checking internet connectivity to npm registry...");
    if ccmFs::cmdExists("curl") {
        let check = Command::new("curl")
            .args(["-I", "-s", "--max-time", "5", "https://registry.npmjs.org"])
            .output();
        match check {
            Ok(o) if o.status.success() => {
                display::success("Internet connection to npm registry is active.");
            }
            _ => {
                display::warn("Failed to connect to registry.npmjs.org. Installation or updates might fail.");
            }
        }
    }

    display::divider();
    if healthy {
        display::success("Environment is healthy for running and managing Claude Code!");
    } else {
        display::error("Some issues were detected. Check output details above.");
    }

    Ok(())
}
