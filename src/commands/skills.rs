use crate::utils::display;
use inquire::MultiSelect;
use std::process::Command;

pub fn runSkills(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Manage Skills for {}", assistant));

    if assistant.to_lowercase() == "claude" {
        return runClaudeSkills();
    }

    if assistant.to_lowercase() == "opencode" {
        return runOpenCodeSkills();
    }

    display::warn("Skills command is currently only supported for 'claude' and 'opencode' assistants.");
    Ok(())
}

fn checkNpx() -> crate::Result<()> {
    let output = Command::new("npx")
        .arg("--version")
        .output()
        .map_err(|_| crate::Error::other("npx not found. Install Node.js from https://nodejs.org"))?;

    if !output.status.success() {
        return Err(crate::Error::other(
            "npx is not available. Install Node.js from https://nodejs.org",
        ));
    }

    Ok(())
}

fn runClaudeSkills() -> crate::Result<()> {
    checkNpx()?;

    display::step("Launching antigravity-awesome-skills for Claude Code...");

    let status = Command::new("npx")
        .args(["--yes", "antigravity-awesome-skills", "--claude"])
        .status()
        .map_err(|e| crate::Error::other(format!("Failed to execute npx: {}", e)))?;

    if !status.success() {
        return Err(crate::Error::other(
            "antigravity-awesome-skills exited with an error. Check your network connection and try again.",
        ));
    }

    display::success("Skills installed in .claude/skills/");
    Ok(())
}

fn runOpenCodeSkills() -> crate::Result<()> {
    display::step("Select skill categories and risk levels for OpenCode...");

    let categories_options = vec![
        "development",
        "frontend",
        "backend",
        "devops",
        "security",
        "data",
        "testing",
        "api",
        "design",
        "documentation",
    ];

    let selected_categories = MultiSelect::new("Select skill categories:", categories_options)
        .with_help_message("Space to toggle, Enter to confirm, Arrow keys to navigate")
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    if selected_categories.is_empty() {
        display::warn("No categories selected. Aborting.");
        return Ok(());
    }

    let risk_options = vec!["none", "safe", "moderate", "high"];

    let selected_risks = MultiSelect::new("Select risk levels:", risk_options)
        .with_help_message("Space to toggle, Enter to confirm, Arrow keys to navigate")
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    if selected_risks.is_empty() {
        display::warn("No risk levels selected. Aborting.");
        return Ok(());
    }

    checkNpx()?;

    let categories = selected_categories.join(",");
    let risks = selected_risks.join(",");

    display::step(&format!(
        "Installing skills with categories: {}, risk levels: {}...",
        categories, risks
    ));

    let status = Command::new("npx")
        .args([
            "--yes",
            "antigravity-awesome-skills",
            "--path",
            ".agents/skills",
            "--category",
            &categories,
            "--risk",
            &risks,
        ])
        .status()
        .map_err(|e| crate::Error::other(format!("Failed to execute npx: {}", e)))?;

    if !status.success() {
        return Err(crate::Error::other(
            "antigravity-awesome-skills exited with an error. Check your network connection and try again.",
        ));
    }

    display::success("Skills installed in .agents/skills/");
    display::info("OpenCode discovers these skills automatically via the built-in skill tool.");
    Ok(())
}
