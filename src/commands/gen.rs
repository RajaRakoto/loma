use crate::utils::display;
use crate::commands::gen_interactive;
use std::fs;
use std::path::Path;

pub fn runGen(assistant: &str) -> crate::Result<()> {
    let assistant_lower = assistant.to_lowercase();
    if assistant_lower == "claude" {
        return gen_interactive::promptAndGenerateClaude();
    }

    if assistant_lower != "copilot" && assistant_lower != "copilot.md" {
        return Err(crate::Error::validation(format!(
            "Unknown or unsupported assistant/template: {}. Loma gen only supports 'claude' or 'copilot'.",
            assistant
        )));
    }

    let targetName = if assistant_lower.ends_with(".md") {
        assistant.to_uppercase()
    } else {
        format!("{}.md", assistant.to_uppercase())
    };

    println!("# Loma Guideline Generator\n");
    println!("- **Target file:** `{}`", targetName);

    let targetPath = Path::new(&targetName);

    if !targetPath.exists() {
        if !display::confirm(&format!("File '{}' does not exist. Create it?", targetName)) {
            println!("\n> **Notice:** File creation aborted by user.");
            return Ok(());
        }
    } else {
        if !display::confirm(&format!("File '{}' already exists. Overwrite it?", targetName)) {
            println!("\n> **Notice:** Overwrite aborted by user. Existing file preserved.");
            return Ok(());
        }
    }

    match gen_interactive::promptAndGenerate()? {
        Some(markdown) => {
            if markdown.is_empty() {
                println!("\n> **Notice:** No elements selected. Nothing to write.");
            } else {
                fs::write(targetPath, &markdown)?;
                println!("\n**Success:** Guidelines successfully generated and written to `{}`.", targetName);
            }
        }
        None => {
            println!("\n> **Notice:** Generation canceled by user.");
        }
    }

    Ok(())
}
