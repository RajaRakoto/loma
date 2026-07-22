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

    // 6. Check OpenCode installation (if applicable)
    display::step("Checking OpenCode...");
    if lomaFs::opencodeIsInstalled() {
        let binaryPath = lomaFs::getOpenCodeBinary();
        display::success(&format!("OpenCode binary found: {}", binaryPath));
        let versionOutput = std::process::Command::new(&binaryPath).arg("--version").output();
        if let Ok(o) = versionOutput {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            display::info(&format!("OpenCode version: {}", ver));
        }
    } else {
        display::warn("OpenCode is not installed. Run 'loma install opencode' to install it.");
    }

    // 7. Check AGENTS.md
    display::step("Checking AGENTS.md...");
    let agents_md = std::path::Path::new("AGENTS.md");
    if agents_md.exists() {
        display::success("AGENTS.md is present.");
    } else {
        display::info("AGENTS.md not found. Run 'loma init opencode' to create one.");
    }

    // 8. Check OpenCode global config
    display::step("Checking OpenCode global configuration...");
    if let Some(globalDir) = lomaFs::getAssistantGlobalDir("opencode") {
        let configFile = globalDir.join("opencode.json");
        if configFile.exists() {
            if let Ok(content) = std::fs::read_to_string(&configFile) {
                if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                    display::success("opencode.json is valid JSON.");
                } else {
                    display::error("opencode.json is not valid JSON!");
                    healthy = false;
                }
            }
        } else {
            display::info("Global opencode.json not found. Run 'loma optimize opencode' to create one.");
        }
    }

    // 9. Check internet connectivity to npm registry
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
