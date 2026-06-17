use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::process::Command;
use inquire::Select;

pub fn runUsage() -> crate::Result<()> {
    display::title("Usage & Metrics Report");

    display::info("ccusage reads local logs of AI assistants and compiles consumption reports.");
    
    let report_types = vec![
        "Standard (today's report)",
    ];

    let _choice = Select::new("Choose report type to execute:", report_types)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

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
