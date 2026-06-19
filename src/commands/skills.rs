use crate::utils::display;
use crate::utils::fs as lomaFs;
use std::fs;
use inquire::MultiSelect;
use serde_json::Value;

pub fn runSkills(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Manage Skills for {}", assistant));

    if assistant.to_lowercase() != "claude" {
        display::warn("Skills command is currently only supported for the 'claude' assistant.");
        return Ok(());
    }

    let assistant_dir = lomaFs::getAssistantDir(assistant);
    if !assistant_dir.exists() {
        display::error("Native Claude directory (.claude/) does not exist. Run 'loma init' first.");
        return Err(crate::Error::other("Missing Native Claude directory"));
    }

    let skills_dir = assistant_dir.join("skills");
    fs::create_dir_all(&skills_dir)?;

    // Load skills from JSON
    let skills_val: Value = serde_json::from_str(include_str!("../json/skills.json")).unwrap_or_else(|_| serde_json::json!({}));
    let Some(skills_obj) = skills_val.as_object() else {
        return Err(crate::Error::other("Failed to load skills database."));
    };

    let mut options = Vec::new();
    let mut keys = Vec::new();
    for (key, val) in skills_obj {
        if let Some(title) = val["title"].as_str() {
            let desc = val["description"].as_str().unwrap_or("");
            options.push(format!("{} - {}", title, desc));
            keys.push(key.clone());
        }
    }

    if options.is_empty() {
        display::info("No skills available.");
        return Ok(());
    }

    // Read currently active skills to pre-select them in the menu
    let mut default_selections = Vec::new();
    for (idx, key) in keys.iter().enumerate() {
        let skill_file_name = format!("{}.md", key);
        let skill_path = skills_dir.join(&skill_file_name);
        if skill_path.exists() {
            default_selections.push(idx);
        }
    }

    // Interactive prompt
    let multi_select_prompt = MultiSelect::new("Select skills to enable/inject in this project:", options.clone());
    let prompt_with_defaults = if !default_selections.is_empty() {
        multi_select_prompt.with_default(&default_selections)
    } else {
        multi_select_prompt
    };

    let selected_choices = prompt_with_defaults
        .with_help_message("Space to select, Enter to confirm, Arrow keys to navigate")
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    // Map selected choices back to their keys
    let selected_keys: Vec<String> = selected_choices.iter().map(|choice| {
        let idx = options.iter().position(|opt| opt == choice).unwrap();
        keys[idx].clone()
    }).collect();

    // Enable/disable skills accordingly
    for key in &keys {
        let skill_file_name = format!("{}.md", key);
        let skill_path = skills_dir.join(&skill_file_name);
        
        if selected_keys.contains(key) {
            let content = skills_obj[key]["content"].as_str().unwrap_or("");
            fs::write(&skill_path, content)?;
            display::success(&format!("Enabled skill: {}", skill_file_name));
        } else if skill_path.exists() {
            fs::remove_file(&skill_path)?;
            display::warn(&format!("Removed disabled skill: {}", skill_file_name));
        }
    }

    display::divider();
    display::success("Skills sync completed successfully!");
    Ok(())
}
