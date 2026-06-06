use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::process::Command;
use inquire::Select;

pub fn runUsage(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Usage & Metrics Report ({})", assistant));

    display::info("ccusage reads local logs of Claude Code and compiles consumption reports.");
    
    let report_types = vec![
        "Standard (today's report)",
        "Daily breakdown",
        "Session breakdown",
        "5-Hour blocks breakdown",
        "Cancel",
    ];

    let choice = Select::new("Choose report type to execute:", report_types)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    if choice == "Cancel" {
        return Ok(());
    }

    let arg = match choice {
        "Daily breakdown" => Some("daily"),
        "Session breakdown" => Some("session"),
        "5-Hour blocks breakdown" => Some("blocks"),
        _ => None,
    };

    let runner = if lomaFs::cmdExists("npx") {
        Some("npx")
    } else if lomaFs::cmdExists("bunx") {
        Some("bunx")
    } else {
        None
    };

    match runner {
        Some(cmd) => {
            let mut command = Command::new(cmd);
            command.arg("ccusage");
            if let Some(a) = arg {
                command.arg(a);
            }
            let status = command.status()?;
            if !status.success() {
                display::error(&format!("Failed to run '{} ccusage'. Make sure your JS runtime is functional.", cmd));
            }
        }
        None => {
            display::error("Neither npx nor bunx was found on PATH. Please install Node.js/npm or Bun.");
        }
    }
    
    Ok(())
}
