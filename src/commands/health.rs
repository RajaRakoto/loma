use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::fs;
use std::process::Command;

pub fn runHealth(assistant: &str) -> crate::Result<()> {
    display::title(&format!("{} Environment Health Check", assistant));

    if assistant != "claude" {
        display::info(&format!("Health check for '{}' is not implemented yet.", assistant));
        return Ok(());
    }

    let mut healthy = true;

    // 1. Check Node.js & npm
    display::step("Checking Node.js & npm...");
    if lomaFs::requireNpm().is_ok() {
        display::success("Node.js >= 18 and npm are present and healthy.");
    } else {
        display::error("Node.js or npm validation failed.");
        healthy = false;
    }

    // 2. Check curl
    display::step("Checking curl...");
    if lomaFs::cmdExists("curl") {
        display::success("curl is available.");
    } else {
        display::warn(
            "curl is missing. Installation requires curl (will try to install it automatically).",
        );
    }

    // 3. Check Fedora/DNF
    display::step("Checking package manager (dnf)...");
    if lomaFs::cmdExists("dnf") {
        display::success("dnf package manager is available.");
    } else {
        display::warn(
            "dnf is not available. System-level integrations (repos, package) might fail.",
        );
    }

    // 4. Check write permissions to local .loma directory
    display::step("Checking .loma directory write permissions...");
    let lomaDir = lomaFs::getLomaDir();
    let testFile = lomaDir.join(format!(".{}_health_test", assistant));
    
    // Ensure the parent exists
    let _ = fs::create_dir_all(&lomaDir);

    match fs::write(&testFile, "test") {
        Ok(_) => {
            let _ = fs::remove_file(&testFile);
            display::success(".loma directory is writable.");
        }
        Err(e) => {
            display::error(&format!(".loma directory is not writable: {}", e));
            healthy = false;
        }
    }

    // 5. Check internet connectivity to registry.npmjs.org
    display::step("Checking internet connectivity to npm registry...");
    if lomaFs::cmdExists("curl") {
        let check = Command::new("curl")
            .args(["-I", "-s", "--max-time", "5", "https://registry.npmjs.org"])
            .output();
        match check {
            Ok(o) if o.status.success() => {
                display::success("Internet connection to npm registry is active.");
            }
            _ => {
                display::warn(
                    "Failed to connect to registry.npmjs.org. Installation or updates might fail.",
                );
            }
        }
    }

    display::divider();
    if healthy {
        display::success(&format!("Environment is healthy for running and managing {}!", assistant));
    } else {
        display::error("Some issues were detected. Check output details above.");
    }

    Ok(())
}
