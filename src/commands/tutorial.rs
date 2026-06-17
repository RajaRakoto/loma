use crate::utils::display;
use inquire::Select;
use serde_json::Value;
use std::fs;

pub fn runTutorial(tool: Option<&str>) -> crate::Result<()> {
    // Load tutorials JSON
    let tutorialsVal = loadTutorialsJson();
    let Some(tutsMap) = tutorialsVal.as_object() else {
        display::error("Failed to load tutorials configuration.");
        return Err(crate::Error::other("Failed to load tutorials configuration"));
    };

    if let Some(toolName) = tool {
        let toolLower = toolName.to_lowercase();
        let foundKey = tutsMap.keys().find(|k| k.to_lowercase() == toolLower);
        if let Some(key) = foundKey {
            if let Some(tut) = tutsMap.get(key) {
                display::title(&format!("Tutorial: {}", toolName));
                printTutorial(tut);
                return Ok(());
            }
        }
        
        display::warn(&format!("No tutorial found for tool: {}", toolName));
        display::info("Available tools:");
    }

    display::title("Loma Tutorials");

    let mut keys = vec![
        "rtk".to_string(),
        "caveman".to_string(),
        "token_optimizer".to_string(),
        "graphify".to_string(),
        "code_review_graph".to_string(),
    ];

    // Add any other keys from JSON to retain compatibility/extensibility
    for k in tutsMap.keys() {
        if !keys.contains(k) {
            keys.push(k.clone());
        }
    }

    // Retain only those keys that exist in the tutorials map
    keys.retain(|k| tutsMap.contains_key(k));

    let mut options = Vec::new();
    let mut valid_keys = Vec::new();
    for k in &keys {
        if let Some(tut) = tutsMap.get(k) {
            if let Some(title) = tut.get("title").and_then(|v| v.as_str()) {
                options.push(title.to_string());
                valid_keys.push(k.clone());
            }
        }
    }

    if options.is_empty() {
        display::info("No tutorials found.");
        return Ok(());
    }

    let choice = Select::new("Choose a tool to view manual setup steps:", options)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    let index = valid_keys.iter().position(|k| {
        if let Some(tut) = tutsMap.get(k) {
            tut.get("title").and_then(|v| v.as_str()) == Some(&choice)
        } else {
            false
        }
    });

    if let Some(idx) = index {
        let key = &valid_keys[idx];
        if let Some(tut) = tutsMap.get(key) {
            display::divider();
            printTutorial(tut);
        }
    }

    Ok(())
}

fn printTutorial(tut: &Value) {
    if let Some(title) = tut.get("title").and_then(|v| v.as_str()) {
        println!("\n\x1b[35;1m✦ {} ✦\x1b[0m", title);
    }
    if let Some(steps) = tut.get("steps").and_then(|v| v.as_array()) {
        for (stepIdx, step) in steps.iter().enumerate() {
            if let Some(stepStr) = step.as_str() {
                println!("  \x1b[32m{}.\x1b[0m {}", stepIdx + 1, stepStr);
            }
        }
    }
    println!();
}

fn loadTutorialsJson() -> Value {
    if let Ok(content) = fs::read_to_string("src/json/tutorials.json") {
        if let Ok(val) = serde_json::from_str(&content) {
            return val;
        }
    }
    let embedded = include_str!("../json/tutorials.json");
    serde_json::from_str(embedded).unwrap_or(Value::Null)
}
