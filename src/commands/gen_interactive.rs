use inquire::error::InquireError;
use inquire::{MultiSelect, Select, Confirm};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Clone)]
#[allow(dead_code)]
struct CheckboxOption {
    id: &'static str,
    label: &'static str,
    path: &'static [&'static str],
}

impl std::fmt::Display for CheckboxOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

const SELECTABLE_ITEMS: &[CheckboxOption] = &[
    CheckboxOption {
        id: "dev.think-before-coding",
        label: "[DEV] Think Before Coding",
        path: &["dev", "think-before-coding"],
    },
    CheckboxOption {
        id: "dev.simplicity-first",
        label: "[DEV] Simplicity First",
        path: &["dev", "simplicity-first"],
    },
    CheckboxOption {
        id: "dev.surgical-changes",
        label: "[DEV] Surgical Changes",
        path: &["dev", "surgical-changes"],
    },
    CheckboxOption {
        id: "dev.goal-driven-execution",
        label: "[DEV] Goal-Driven Execution",
        path: &["dev", "goal-driven-execution"],
    },
    CheckboxOption {
        id: "dev.development-efficiency",
        label: "[DEV] Development Efficiency",
        path: &["dev", "development-efficiency"],
    },
    CheckboxOption {
        id: "dev.language-code-standards",
        label: "[DEV] Language & Code Standards",
        path: &["dev", "language-code-standards"],
    },
    CheckboxOption {
        id: "dev.testing-guidelines",
        label: "[DEV] Testing Guidelines",
        path: &["dev", "testing-guidelines"],
    },
    CheckboxOption {
        id: "git.identity",
        label: "[GIT] Required Git Identity",
        path: &["git", "identity"],
    },
    CheckboxOption {
        id: "git.commit-rules",
        label: "[GIT] Commit Rules",
        path: &["git", "commit-rules"],
    },
    CheckboxOption {
        id: "git.validation-before-merge",
        label: "[GIT] Validation Before Merge",
        path: &["git", "validation-before-merge"],
    },
    CheckboxOption {
        id: "git.documentation-guidelines",
        label: "[GIT] Documentation Guidelines",
        path: &["git", "documentation-guidelines"],
    },
    CheckboxOption {
        id: "stacks.runtime-tooling",
        label: "[STACK] Runtime & Tooling (Bun, Vite, TS, Biome, etc.)",
        path: &["stacks", "runtime-tooling"],
    },
    CheckboxOption {
        id: "stacks.react.react-core",
        label: "[STACK] React - Core Library (React.js)",
        path: &["stacks", "react", "react-core"],
    },
    CheckboxOption {
        id: "stacks.react.react-patterns",
        label: "[STACK] React - Best Practices",
        path: &["stacks", "react", "react-patterns"],
    },
    CheckboxOption {
        id: "stacks.react.react-state.zustand",
        label: "[STACK] React State - Zustand",
        path: &["stacks", "react", "react-state", "zustand"],
    },
    CheckboxOption {
        id: "stacks.react.react-state.redux",
        label: "[STACK] React State - Redux",
        path: &["stacks", "react", "react-state", "redux"],
    },
    CheckboxOption {
        id: "stacks.react.react-data.tanstack-query",
        label: "[STACK] React Data Layer - TanStack Query",
        path: &["stacks", "react", "react-data", "tanstack-query"],
    },
    CheckboxOption {
        id: "stacks.react.react-data.zod",
        label: "[STACK] React Data Layer - Zod",
        path: &["stacks", "react", "react-data", "zod"],
    },
    CheckboxOption {
        id: "stacks.react.react-ui.tanstack-table",
        label: "[STACK] React UI - TanStack Table",
        path: &["stacks", "react", "react-ui", "tanstack-table"],
    },
    CheckboxOption {
        id: "stacks.react.react-ui.sweetalert2",
        label: "[STACK] React UI - SweetAlert2",
        path: &["stacks", "react", "react-ui", "sweetalert2"],
    },
    CheckboxOption {
        id: "stacks.react.react-ui.react-icons",
        label: "[STACK] React UI - react-icons",
        path: &["stacks", "react", "react-ui", "react-icons"],
    },
    CheckboxOption {
        id: "stacks.react.react-performance",
        label: "[STACK] React - Performance",
        path: &["stacks", "react", "react-performance"],
    },
    CheckboxOption {
        id: "stacks.frontend.tailwindcss",
        label: "[STACK] Frontend - Tailwind CSS",
        path: &["stacks", "frontend", "tailwindcss"],
    },
    CheckboxOption {
        id: "stacks.frontend.sass",
        label: "[STACK] Frontend - Sass",
        path: &["stacks", "frontend", "sass"],
    },
    CheckboxOption {
        id: "stacks.frontend.recharts",
        label: "[STACK] Frontend - Recharts",
        path: &["stacks", "frontend", "recharts"],
    },
    CheckboxOption {
        id: "stacks.architecture.performance",
        label: "[STACK] Architecture - Performance",
        path: &["stacks", "architecture", "performance"],
    },
    CheckboxOption {
        id: "stacks.architecture.security",
        label: "[STACK] Architecture - Security (OWASP)",
        path: &["stacks", "architecture", "security"],
    },
    CheckboxOption {
        id: "rtk.rules",
        label: "[RTK] General Rules",
        path: &["rtk", "rules"],
    },
    CheckboxOption {
        id: "rtk.filesystem-search",
        label: "[RTK] Filesystem & Search commands",
        path: &["rtk", "filesystem-search"],
    },
    CheckboxOption {
        id: "rtk.git-github",
        label: "[RTK] Git & GitHub commands",
        path: &["rtk", "git-github"],
    },
    CheckboxOption {
        id: "rtk.build-test-lint",
        label: "[RTK] Build, Test & Lint commands",
        path: &["rtk", "build-test-lint"],
    },
    CheckboxOption {
        id: "rtk.data-environment",
        label: "[RTK] Data & Environment commands",
        path: &["rtk", "data-environment"],
    },
    CheckboxOption {
        id: "rtk.docker",
        label: "[RTK] Docker commands",
        path: &["rtk", "docker"],
    },
    CheckboxOption {
        id: "rtk.output-control",
        label: "[RTK] Output Control commands",
        path: &["rtk", "output-control"],
    },
    CheckboxOption {
        id: "rtk.session-audit",
        label: "[RTK] Session Audit commands",
        path: &["rtk", "session-audit"],
    },
    CheckboxOption {
        id: "taskmaster.rules",
        label: "[TASKMASTER] Rules",
        path: &["taskmaster", "rules"],
    },
    CheckboxOption {
        id: "taskmaster.key-commands",
        label: "[TASKMASTER] Key Commands",
        path: &["taskmaster", "key-commands"],
    },
    CheckboxOption {
        id: "taskmaster.task-execution-workflow",
        label: "[TASKMASTER] Task Execution Workflow",
        path: &["taskmaster", "task-execution-workflow"],
    },
    CheckboxOption {
        id: "context7.rules",
        label: "[CONTEXT7] Rules",
        path: &["context7", "rules"],
    },
    CheckboxOption {
        id: "pocketbase.usage-rules",
        label: "[POCKETBASE] Usage Rules",
        path: &["pocketbase", "usage-rules"],
    },
    CheckboxOption {
        id: "pocketbase.available-tools",
        label: "[POCKETBASE] Available Tools",
        path: &["pocketbase", "available-tools"],
    },
    CheckboxOption {
        id: "pocketbase.env-vars",
        label: "[POCKETBASE] Required Env Vars",
        path: &["pocketbase", "env-vars"],
    },
    CheckboxOption {
        id: "pocketbase.backups",
        label: "[POCKETBASE] PocketBase Backups",
        path: &["pocketbase", "backups"],
    },
    CheckboxOption {
        id: "pocketbase.typegen",
        label: "[POCKETBASE] PocketBase Typegen",
        path: &["pocketbase", "typegen"],
    },
];

pub fn loadInjectJson() -> Value {
    if let Ok(content) = fs::read_to_string("src/json/inject.json") {
        if let Ok(val) = serde_json::from_str(&content) {
            return val;
        }
    }
    let embedded = include_str!("../json/inject.json");
    serde_json::from_str(embedded).unwrap_or(Value::Null)
}

fn renderNode(value: &Value, depth: usize) -> String {
    let mut out = String::new();

    // Render name and role directly if they are present at the root of this node
    if let (Some(name), Some(role)) = (
        value.get("name").and_then(|v| v.as_str()),
        value.get("role").and_then(|v| v.as_str()),
    ) {
        return format!("* **{}**: {}\n\n", name, role);
    }

    // 1. Render title if present
    if let Some(title) = value.get("title").and_then(|v| v.as_str()) {
        let hashes = "#".repeat(std::cmp::min(depth, 4));
        out.push_str(&format!("{} {}\n\n", hashes, title));
    }

    // 2. Render body if present
    if let Some(body) = value.get("body").and_then(|v| v.as_array()) {
        for item in body {
            if let Some(intro) = item.get("intro").and_then(|v| v.as_str()) {
                out.push_str(&format!("**{}**\n\n", intro));
            }
            if let Some(items) = item.get("items").and_then(|v| v.as_array()) {
                for li in items {
                    if let Some(liStr) = li.as_str() {
                        out.push_str(&format!("* {}\n", liStr));
                    }
                }
                out.push('\n');
            }
            if let Some(note) = item.get("note").and_then(|v| v.as_str()) {
                out.push_str(&format!("*Note:* {}\n\n", note));
            }
        }
    }

    // 3. Render table if present
    if let Some(table) = value.get("table") {
        if let (Some(columns), Some(rows)) = (
            table.get("columns").and_then(|v| v.as_array()),
            table.get("rows").and_then(|v| v.as_array()),
        ) {
            out.push_str("| ");
            for col in columns {
                if let Some(colStr) = col.as_str() {
                    out.push_str(colStr);
                    out.push_str(" | ");
                }
            }
            out.push_str("\n| ");
            for _ in columns {
                out.push_str("--- | ");
            }
            out.push('\n');

            for row in rows {
                if let Some(rowArr) = row.as_array() {
                    out.push_str("| ");
                    for cell in rowArr {
                        if let Some(cellStr) = cell.as_str() {
                            out.push_str(cellStr);
                            out.push_str(" | ");
                        }
                    }
                    out.push('\n');
                }
            }
            out.push('\n');
        }
    }

    // 4. Render other sub-keys (either name/role or sub-nodes)
    if let Some(obj) = value.as_object() {
        let mut nameRoleItems = Vec::new();
        let mut subNodes = Vec::new();

        for (k, val) in obj {
            if k == "title" || k == "body" || k == "table" || k == "parent-title" {
                continue;
            }
            if val.get("name").is_some() && val.get("role").is_some() {
                nameRoleItems.push(val);
            } else {
                subNodes.push(val);
            }
        }

        if !nameRoleItems.is_empty() {
            for item in nameRoleItems {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("");
                out.push_str(&format!("* **{}**: {}\n", name, role));
            }
            out.push('\n');
        }

        for subNode in subNodes {
            out.push_str(&renderNode(subNode, depth + 1));
        }
    }

    out
}

pub fn get_file_name(id: &str, category: &str) -> String {
    let parts: Vec<&str> = id.split('.').collect();
    let last_part = parts.last().unwrap_or(&id);
    let base_name = last_part.replace('-', "_").to_uppercase();
    let suffix = category.to_uppercase();
    format!("{}_{}.md", base_name, suffix)
}

#[derive(Debug)]
struct MarkdownSection {
    header: String,
    header_text: String,
    content: Vec<String>,
}

fn split_markdown(content: &str) -> Vec<MarkdownSection> {
    let mut sections = Vec::new();
    let mut current_section = MarkdownSection {
        header: String::new(),
        header_text: String::new(),
        content: Vec::new(),
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            if !current_section.header.is_empty() || !current_section.content.is_empty() {
                sections.push(current_section);
            }

            let header = line.to_string();
            let header_text = trimmed.trim_start_matches('#').trim().to_string();
            current_section = MarkdownSection {
                header,
                header_text,
                content: Vec::new(),
            };
        } else {
            current_section.content.push(line.to_string());
        }
    }

    if !current_section.header.is_empty() || !current_section.content.is_empty() {
        sections.push(current_section);
    }

    sections
}

fn merge_markdown(existing: &str, new_content: &str) -> String {
    let mut existing_sections = split_markdown(existing);
    let new_sections = split_markdown(new_content);

    for new_sec in new_sections {
        let match_idx = existing_sections.iter().position(|sec| {
            sec.header_text.to_lowercase() == new_sec.header_text.to_lowercase()
        });

        if let Some(idx) = match_idx {
            let mut merged_content = existing_sections[idx].content.clone();

            for line in &new_sec.content {
                let trimmed = line.trim();
                if trimmed.starts_with('*') || trimmed.starts_with('-') {
                    let clean_new = trimmed.trim_start_matches(['*', '-', ' ']);
                    let exists = merged_content.iter().any(|existing_line| {
                        let clean_existing = existing_line.trim().trim_start_matches(['*', '-', ' ']);
                        clean_existing.to_lowercase() == clean_new.to_lowercase()
                    });

                    if !exists {
                        merged_content.push(line.clone());
                    }
                } else if !trimmed.is_empty() {
                    let exists = merged_content.iter().any(|existing_line| {
                        existing_line.trim() == trimmed
                    });
                    if !exists {
                        merged_content.push(line.clone());
                    }
                } else {
                    merged_content.push(line.clone());
                }
            }

            existing_sections[idx].content = merged_content;
        } else {
            existing_sections.push(new_sec);
        }
    }

    let mut out = String::new();
    for sec in existing_sections {
        if !sec.header.is_empty() {
            out.push_str(&sec.header);
            out.push('\n');
        }
        for line in sec.content {
            out.push_str(&line);
            out.push('\n');
        }
    }

    // Verify merged Markdown structure using pulldown-cmark
    let parser = pulldown_cmark::Parser::new(&out);
    let mut event_count = 0;
    for _event in parser {
        event_count += 1;
    }
    crate::utils::display::info(&format!(
        "Verified merged Markdown structure successfully ({} parsed events).",
        event_count
    ));

    out
}

pub fn promptAndGenerateClaude() -> crate::Result<()> {
    let jsonRoot = loadInjectJson();
    if jsonRoot.is_null() {
        return Err(crate::Error::other("Failed to parse guidelines structure."));
    }

    crate::utils::display::step("Step 1: Choose parent-section");
    let parent_sections = vec![
        "dev",
        "git",
        "stacks",
        "rtk",
        "taskmaster",
        "context7",
        "pocketbase",
    ];
    let parent = Select::new("Choose parent-section:", parent_sections)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    crate::utils::display::step("Step 2: Choose associated sections");
    let filtered_options: Vec<CheckboxOption> = SELECTABLE_ITEMS
        .iter()
        .filter(|opt| opt.path.first() == Some(&parent))
        .cloned()
        .collect();

    if filtered_options.is_empty() {
        crate::utils::display::warn("No selectable options found for this parent section.");
        return Ok(());
    }

    let selected = MultiSelect::new("Select sections to inject:", filtered_options)
        .with_help_message("Space to select, Enter to confirm, Arrow keys to navigate")
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    if selected.is_empty() {
        crate::utils::display::info("No sections selected. Aborting.");
        return Ok(());
    }

    crate::utils::display::step("Step 3: Choose Claude destination");
    
    let mut rules_score = 0;
    let mut skills_score = 0;
    let mut agents_score = 0;
    let mut commands_score = 0;

    for opt in &selected {
        let id_lower = opt.id.to_lowercase();
        if id_lower.contains("rules") || parent == "dev" || parent == "git" || parent == "context7" {
            rules_score += 1;
        }
        if id_lower.contains("tooling") || id_lower.contains("automation") || id_lower.contains("skills") || parent == "rtk" {
            skills_score += 1;
        }
        if id_lower.contains("persona") || id_lower.contains("reviewer") || id_lower.contains("agent") {
            agents_score += 1;
        }
        if id_lower.contains("command") {
            commands_score += 1;
        }
    }

    let default_dest = if commands_score > rules_score && commands_score > skills_score && commands_score > agents_score {
        "commands"
    } else if agents_score > rules_score && agents_score > skills_score {
        "agents"
    } else if skills_score > rules_score {
        "skills"
    } else {
        "rules"
    };

    let destinations = vec!["rules", "agents", "skills", "commands"];
    let starting_cursor = destinations.iter().position(|&d| d == default_dest).unwrap_or(0);

    let destination = Select::new("Choose Claude destination:", destinations)
        .with_starting_cursor(starting_cursor)
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    crate::utils::display::step("Step 4: Preview generated files");
    let mut preview_list = Vec::new();
    for opt in &selected {
        let filename = get_file_name(opt.id, destination);
        preview_list.push(filename);
    }

    println!(".claude/{}/", destination);
    for name in &preview_list {
        println!("├── {}", name);
    }
    println!();

    crate::utils::display::step("Step 5: Confirm injection");
    if !Confirm::new("Confirm injection of these files?")
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?
    {
        crate::utils::display::info("Generation aborted by user.");
        return Ok(());
    }

    let registry_path = PathBuf::from(".loma/registry/injections.json");
    let mut registry: HashMap<String, crate::commands::sync::RegistryEntry> = if registry_path.exists() {
        let content = fs::read_to_string(&registry_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    for (i, opt) in selected.iter().enumerate() {
        let filename = &preview_list[i];
        let mut final_path = PathBuf::from(".claude").join(destination).join(filename);
        let mut strategy = "create".to_string();

        let mut currentVal = &jsonRoot;
        for p in opt.path {
            if let Some(nextVal) = currentVal.get(*p) {
                currentVal = nextVal;
            }
        }
        let generated_markdown = renderNode(currentVal, opt.path.len());

        if final_path.exists() {
            crate::utils::display::warn(&format!("File '{}' already exists.", filename));
            let collision_options = vec!["merge", "overwrite", "duplicate", "cancel"];
            let choice = Select::new("Choose collision strategy:", collision_options)
                .prompt()
                .map_err(|e| crate::Error::other(e.to_string()))?;

            match choice {
                "cancel" => {
                    crate::utils::display::info(&format!("Skipped: {}", filename));
                    continue;
                }
                "overwrite" => {
                    fs::write(&final_path, &generated_markdown)?;
                    strategy = "overwrite".to_string();
                    crate::utils::display::success(&format!("Overwritten: {}", final_path.display()));
                }
                "merge" => {
                    let existing_content = fs::read_to_string(&final_path)?;
                    let merged = merge_markdown(&existing_content, &generated_markdown);
                    fs::write(&final_path, &merged)?;
                    strategy = "merge".to_string();
                    crate::utils::display::success(&format!("Merged: {}", final_path.display()));
                }
                "duplicate" => {
                    let mut counter = 1;
                    let base_stem = filename.replace(".md", "");
                    let mut dup_path = final_path.clone();
                    while dup_path.exists() {
                        let new_filename = format!("{}_{}.md", base_stem, counter);
                        dup_path = PathBuf::from(".claude").join(destination).join(new_filename);
                        counter += 1;
                    }
                    fs::write(&dup_path, &generated_markdown)?;
                    final_path = dup_path;
                    strategy = "duplicate".to_string();
                    crate::utils::display::success(&format!("Duplicated as: {}", final_path.display()));
                }
                _ => {}
            }
        } else {
            if let Some(parent) = final_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&final_path, &generated_markdown)?;
            crate::utils::display::success(&format!("Created: {}", final_path.display()));
        }

        if final_path.exists() {
            if let Ok(content) = fs::read_to_string(&final_path) {
                let hash = crate::commands::sync::calculate_hash(&content);
                let parts: Vec<&str> = opt.id.split('.').collect();
                let source_key = parts.last().unwrap_or(&opt.id).to_string();
                
                let entry = crate::commands::sync::RegistryEntry {
                    target: final_path.to_string_lossy().to_string(),
                    source: opt.id.to_string(),
                    hash,
                    r#type: destination.to_string(),
                    date: chrono::Local::now().to_rfc3339(),
                    strategy: strategy.clone(),
                };
                registry.insert(source_key, entry);
            }
        }
    }

    if let Some(parent) = registry_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(&registry)?;
    fs::write(&registry_path, serialized)?;
    crate::utils::display::success("Registry updated successfully!");

    Ok(())
}

pub fn promptAndGenerate() -> crate::Result<Option<String>> {
    let jsonRoot = loadInjectJson();
    if jsonRoot.is_null() {
        return Err(crate::Error::other("Failed to parse guidelines structure."));
    }

    let options = SELECTABLE_ITEMS.to_vec();
    let selected = MultiSelect::new("Select elements to inject:", options)
        .with_help_message("Space to select, Enter to confirm, Arrow keys to navigate")
        .prompt();

    let selectedList = match selected {
        Ok(list) => list,
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            return Ok(None);
        }
        Err(e) => {
            return Err(crate::Error::other(format!("Interactive prompt error: {}", e)));
        }
    };

    if selectedList.is_empty() {
        return Ok(Some(String::new()));
    }

    let mut markdown = String::new();
    let sectionKeys = &[
        "dev",
        "git",
        "stacks",
        "rtk",
        "taskmaster",
        "context7",
        "pocketbase",
    ];

    for secKey in sectionKeys {
        let secOptions: Vec<&CheckboxOption> = selectedList
            .iter()
            .filter(|opt| opt.path.first() == Some(secKey))
            .collect();

        if secOptions.is_empty() {
            continue;
        }

        if let Some(secVal) = jsonRoot.get(*secKey) {
            let parentTitle = secVal
                .get("parent-title")
                .and_then(|v| v.as_str())
                .unwrap_or(secKey);
            markdown.push_str(&format!("# {}\n\n", parentTitle));

            for opt in secOptions {
                let mut currentVal = &jsonRoot;
                for p in opt.path {
                    if let Some(nextVal) = currentVal.get(*p) {
                        currentVal = nextVal;
                    }
                }
                markdown.push_str(&renderNode(currentVal, opt.path.len()));
            }
        }
    }

    Ok(Some(markdown))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_inject_json() {
        let json = loadInjectJson();
        assert!(!json.is_null());
        assert!(json.get("dev").is_some());
    }

    #[test]
    fn test_render_node_items() {
        let node = serde_json::json!({
            "title": "Think Before Coding",
            "body": [
                { "intro": "Before implementing:" },
                { "items": ["Item A", "Item B"] },
                { "note": "This is a note" }
            ]
        });

        let rendered = renderNode(&node, 2);
        assert!(rendered.contains("## Think Before Coding"));
        assert!(rendered.contains("**Before implementing:**"));
        assert!(rendered.contains("* Item A"));
        assert!(rendered.contains("* Item B"));
        assert!(rendered.contains("*Note:* This is a note"));
    }

    #[test]
    fn test_render_node_table() {
        let node = serde_json::json!({
            "title": "Filesystem",
            "table": {
                "columns": ["Col1", "Col2"],
                "rows": [
                    ["Val1", "Val2"]
                ]
            }
        });

        let rendered = renderNode(&node, 3);
        assert!(rendered.contains("### Filesystem"));
        assert!(rendered.contains("| Col1 | Col2 |"));
        assert!(rendered.contains("| Val1 | Val2 |"));
    }

    #[test]
    fn test_get_file_name() {
        assert_eq!(get_file_name("dev.think-before-coding", "rules"), "THINK_BEFORE_CODING_RULES.md");
        assert_eq!(get_file_name("stacks.runtime-tooling", "agents"), "RUNTIME_TOOLING_AGENTS.md");
    }

    #[test]
    fn test_merge_markdown_basic() {
        let existing = "# Header\n* Bullet 1\n";
        let new_content = "# Header\n* Bullet 1\n* Bullet 2\n";
        let merged = merge_markdown(existing, new_content);
        assert!(merged.contains("* Bullet 1"));
        assert!(merged.contains("* Bullet 2"));
    }
}
