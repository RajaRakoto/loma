use crate::utils::display;
use std::fs;
use std::path::Path;

pub fn runGen() -> crate::Result<()> {
    display::title("Generate Configuration Templates");

    // 1. Generate systemd service template
    display::step("Generating systemd service template (claude-code.service)...");
    let servicePath = Path::new("claude-code.service");
    if servicePath.exists() {
        display::info("claude-code.service file already exists in current directory.");
    } else {
        let serviceContent = r#"[Unit]
Description=Claude Code Service Daemon
After=network.target

[Service]
Type=simple
User=raja
ExecStart=/home/raja/.local/bin/claude daemon
Restart=on-failure
Environment=PATH=/home/raja/.local/bin:/usr/local/bin:/usr/bin:/bin

[Install]
WantedBy=multi-user.target
"#;
        match fs::write(servicePath, serviceContent) {
            Ok(_) => display::success("Generated claude-code.service template successfully."),
            Err(e) => {
                display::error(&format!("Failed to write service template: {}", e));
                return Err(crate::Error::other("Failed to write service template"));
            }
        }
    }

    // 2. Generate settings.json template
    display::step("Generating default settings.json template...");
    let settingsPath = Path::new("settings.template.json");
    if settingsPath.exists() {
        display::info("settings.template.json file already exists.");
    } else {
        let settingsContent = r#"{
  "theme": "dark",
  "autoCompactWindow": 190000,
  "telemetry": false,
  "defaultModel": "claude-3-5-sonnet",
  "editor": "nano"
}
"#;
        match fs::write(settingsPath, settingsContent) {
            Ok(_) => display::success("Generated settings.template.json successfully."),
            Err(e) => {
                display::error(&format!("Failed to write settings template: {}", e));
                return Err(crate::Error::other("Failed to write settings template"));
            }
        }
    }

    display::divider();
    display::success("Generation templates created successfully.");

    Ok(())
}
