use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::fs;
use std::process::Command;

pub fn runDoctor() -> crate::Result<()> {
    display::title("Loma Global Environment Health Check");

    let mut healthy = true;

    // 1. Check Node.js & npm
    display::step("Checking Node.js & npm...");
    if lomaFs::requireNpm().is_ok() {
        display::success("Node.js >= 18 and npm are present and healthy.");
    } else {
        display::warn(
            "Node.js or npm validation failed. Node.js-based integrations might not work.",
        );
        healthy = false;
    }

    // 2. Check curl
    display::step("Checking curl...");
    if lomaFs::cmdExists("curl") {
        display::success("curl is available.");
    } else {
        display::warn("curl is missing. Installation scripts requiring curl will fail.");
    }

    // 3. Check powershell on Windows
    if cfg!(windows) {
        display::step("Checking PowerShell...");
        if lomaFs::cmdExists("powershell") {
            display::success("PowerShell is available.");
        } else {
            display::error("PowerShell is not available. Script executions on Windows will fail.");
            healthy = false;
        }
    }

    // 4. Check git
    display::step("Checking git...");
    if lomaFs::cmdExists("git") {
        display::success("git is available.");
    } else {
        display::warn("git is missing. Project integration might be limited.");
    }

    // 5. Check write permissions to local .loma directory
    display::step("Checking .loma directory write permissions...");
    let lomaDir = lomaFs::getLomaDir();
    let testFile = lomaDir.join(".health_write_test");

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

    // 6. Check internet connectivity to npm registry
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
    } else if cfg!(windows) && lomaFs::cmdExists("powershell") {
        let check = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "try { $r = Invoke-WebRequest -Uri https://registry.npmjs.org -Method Head -TimeoutSec 5; exit 0 } catch { exit 1 }",
            ])
            .status();
        match check {
            Ok(s) if s.success() => {
                display::success("Internet connection to npm registry is active (PowerShell).");
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
        display::success("Environment is healthy for running Loma CLI!");
    } else {
        display::warn(
            "Some global issues or missing dependencies were detected. Check details above.",
        );
    }

    Ok(())
}
