use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::process::Command;
use inquire::{Select, Confirm};

pub fn runUsage() -> crate::Result<()> {
    display::title("Usage & Metrics Report");

    let report_types = vec![
        "RTK Gains",
        "CCUsage",
    ];

    let choice = Select::new("Choose option:", report_types)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    match choice {
        "RTK Gains" => {
            if lomaFs::cmdExists("rtk") {
                display::info("Executing 'rtk gain'...");
                let mut command = Command::new("rtk");
                command.arg("gain");
                let status = command.status()?;
                if !status.success() {
                    display::error("rtk gain execution failed.");
                }
            } else {
                display::error("RTK (Rust Token Kill) is not installed globally.");
                display::info("To install RTK globally, you can run the following installation script:");
                display::info("  curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh");
                
                let install = Confirm::new("Would you like to install RTK globally now?")
                    .with_default(false)
                    .prompt()
                    .unwrap_or(false);

                if install {
                    display::step("Installing RTK globally...");
                    let status = Command::new("sh")
                        .arg("-c")
                        .arg("curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh | sh")
                        .status()?;
                    
                    if status.success() {
                        display::success("RTK was successfully installed globally!");
                        display::info("Please note: you might need to restart your terminal or shell session to make 'rtk' accessible from any directory.");
                        
                        // Check if we can run it immediately in the current process
                        if lomaFs::cmdExists("rtk") {
                            display::info("Executing 'rtk gain'...");
                            let mut command = Command::new("rtk");
                            command.arg("gain");
                            let _ = command.status();
                        } else {
                            display::info("You can execute 'rtk gain' after restarting your shell session.");
                        }
                    } else {
                        display::error("Failed to install RTK globally.");
                    }
                }
            }
        }
        "CCUsage" => {
            // Step 1: Select source/model
            let sources = vec![
                "Claude",
                "Codex",
                "OpenCode",
                "Amp",
                "Droid",
                "Codebuff",
                "Hermes",
                "Goose",
                "OpenClaw",
                "Kilo",
                "Kimi",
                "Qwen",
                "Copilot",
                "Gemini",
            ];
            let source_choice = Select::new("Step 1 — Sélection de la source/modèle:", sources)
                .prompt()
                .map_err(|e| crate::Error::other(e.to_string()))?;

            // Step 2: Select period
            let periods = vec![
                "Daily",
                "Weekly",
                "Monthly",
                "Session",
                "Blocks",
            ];
            let period_choice = Select::new("Step 2 — Sélection de la période:", periods)
                .prompt()
                .map_err(|e| crate::Error::other(e.to_string()))?;

            // Step 3: Select format
            let formats = vec![
                "Standard",
                "Compact",
            ];
            let format_choice = Select::new("Step 3 — Sélection du format:", formats)
                .prompt()
                .map_err(|e| crate::Error::other(e.to_string()))?;

            // Step 4: Select scope
            let scopes = vec![
                "Projet courant (répertoire où Loma a été lancé)",
                "Global",
            ];
            let scope_choice = Select::new("Step 4 — Sélection de la portée:", scopes)
                .prompt()
                .map_err(|e| crate::Error::other(e.to_string()))?;

            // Build arguments
            let mut args = Vec::new();
            
            // Auto-detect runner or command
            let (runner_cmd, has_ccusage_arg) = if lomaFs::cmdExists("ccusage") {
                ("ccusage", false)
            } else if lomaFs::cmdExists("bunx") {
                ("bunx", true)
            } else if lomaFs::cmdExists("npx") {
                ("npx", true)
            } else {
                // Default fallback
                ("ccusage", false)
            };

            if has_ccusage_arg {
                args.push("ccusage".to_string());
            }

            // Command syntax: ccusage <source> <period> [options]
            args.push(source_choice.to_lowercase());
            args.push(period_choice.to_lowercase());

            // Format flag
            if format_choice == "Compact" {
                args.push("--compact".to_string());
            }

            // Scope flag
            if scope_choice.starts_with("Projet") {
                let current_dir = std::env::current_dir()?;
                let project_name = current_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("project");
                args.push("--project".to_string());
                args.push(project_name.to_string());
            }

            // Step 5: Execute
            let full_command_str = format!("{} {}", runner_cmd, args.join(" "));
            display::step(&format!("Génération et exécution automatique de la commande: {}", full_command_str));

            let mut command = Command::new(runner_cmd);
            command.args(&args);
            let status = command.status()?;
            if !status.success() {
                display::error("ccusage execution failed.");
            }
        }
        _ => {}
    }

    Ok(())
}
