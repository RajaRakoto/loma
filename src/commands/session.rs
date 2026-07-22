use crate::utils::display;
use crate::utils::fs as lomaFs;
use inquire::{MultiSelect, Select};
use std::process::Command;

struct SessionInfo {
    id: String,
    title: String,
    updated: String,
}

pub fn runSession(assistant: &str) -> crate::Result<()> {
    match assistant.to_lowercase().as_str() {
        "opencode" => runOpenCodeSession(),
        _ => Err(crate::Error::validation(format!(
            "Unknown assistant '{}'. Supported: opencode",
            assistant
        ))),
    }
}

fn runOpenCodeSession() -> crate::Result<()> {
    if !lomaFs::opencodeIsInstalled() {
        display::error("OpenCode not installed.");
        display::info("Install with: loma install opencode");
        return Err(crate::Error::other("OpenCode not found"));
    }

    loop {
        let sessions = match fetchSessions() {
            Ok(s) => s,
            Err(e) => {
                display::error(&format!("Failed to fetch sessions: {}", e));
                if !display::confirm("Retry?") {
                    break;
                }
                continue;
            }
        };

        if sessions.is_empty() {
            display::info("No active sessions found.");
            break;
        }

        display::title(&format!(
            "OpenCode Session Manager ({} session{})",
            sessions.len(),
            if sessions.len() > 1 { "s" } else { "" }
        ));

        displaySessionsTable(&sessions);

        let choices = vec![
            "Delete a session",
            "Delete multiple sessions",
            "Delete ALL sessions",
            "Refresh list",
            "Exit",
        ];

        let choice = Select::new("Select action:", choices)
            .with_help_message("Enter to confirm, Arrow keys to navigate")
            .prompt()
            .map_err(|e| crate::Error::other(e.to_string()))?;

        match choice {
            "Delete a session" => {
                if let Some(session) = pickSession(&sessions)? {
                    if display::confirm(&format!("Delete '{}'?", session.title)) {
                        deleteSession(&session.id)?;
                        display::success(&format!("Deleted: {}", session.id));
                    } else {
                        display::info("Cancelled.");
                    }
                }
            }
            "Delete multiple sessions" => {
                let selected = pickMultipleSessions(&sessions)?;
                if !selected.is_empty()
                    && display::confirm(&format!("Delete {} session(s)?", selected.len()))
                {
                    let (deleted, errors) = bulkDeleteSessions(&selected);
                    display::success(&format!(
                        "Deleted {}/{} session(s).",
                        deleted,
                        selected.len()
                    ));
                    for (id, err) in &errors {
                        display::error(&format!("Delete failed [{}]: {}", id, err));
                    }
                }
            }
            "Delete ALL sessions" => {
                if display::confirm(&format!(
                    "Delete ALL {} session(s)? This cannot be undone.",
                    sessions.len()
                )) {
                    let (deleted, errors) = bulkDeleteSessions(&sessions);
                    display::success(&format!(
                        "Deleted {}/{} session(s).",
                        deleted,
                        sessions.len()
                    ));
                    for (id, err) in &errors {
                        display::error(&format!("Delete failed [{}]: {}", id, err));
                    }
                    if errors.is_empty() {
                        display::info("No remaining sessions. Returning to main menu.");
                        break;
                    }
                }
            }
            "Refresh list" => continue,
            "Exit" => break,
            _ => {}
        }

        println!();
        if !display::confirm("Continue managing sessions?") {
            break;
        }
    }

    Ok(())
}

fn bulkDeleteSessions(sessions: &[SessionInfo]) -> (usize, Vec<(String, String)>) {
    let mut deleted = 0;
    let mut errors = Vec::new();
    let total = sessions.len();

    for (i, s) in sessions.iter().enumerate() {
        display::info(&format!(
            "[{}/{}] Deleting {}...",
            i + 1,
            total,
            s.id
        ));
        match deleteSession(&s.id) {
            Ok(()) => deleted += 1,
            Err(e) => errors.push((s.id.clone(), e.to_string())),
        }
    }

    (deleted, errors)
}

fn fetchSessions() -> crate::Result<Vec<SessionInfo>> {
    let output = Command::new("opencode")
        .args(["session", "list"])
        .output()
        .map_err(|e| crate::Error::other(format!("Failed to run opencode session list: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::Error::other(format!(
            "opencode session list failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parseSessionsOutput(&stdout))
}

fn parseSessionsOutput(output: &str) -> Vec<SessionInfo> {
    let mut sessions = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    for line in lines.iter().skip(2) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let first_space = trimmed.find(' ').unwrap_or(trimmed.len());
        let id = trimmed[..first_space].trim();
        if !id.starts_with("ses_") {
            continue;
        }

        let rest = trimmed[first_space..].trim();
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        if tokens.len() < 2 {
            continue;
        }

        let last = tokens[tokens.len() - 1];
        let second_last = tokens[tokens.len() - 2];

        if (last == "AM" || last == "PM") && second_last.contains(':') {
            let time_str = format!("{} {}", second_last, last);
            let time_pos = rest.rfind(&time_str).unwrap_or(0);
            let title = rest[..time_pos].trim().to_string();

            sessions.push(SessionInfo {
                id: id.to_string(),
                title,
                updated: time_str,
            });
        }
    }

    sessions
}

fn displaySessionsTable(sessions: &[SessionInfo]) {
    if sessions.is_empty() {
        return;
    }

    println!();
    println!("  {:<30}  {:<38}  {}", "Session ID", "Title", "Updated");
    println!("  {}", "─".repeat(78));
    for s in sessions {
        let title = if s.title.len() > 36 {
            format!("{}...", &s.title[..33])
        } else {
            s.title.clone()
        };
        println!("  {:<30}  {:<38}  {}", s.id, title, s.updated);
    }
    println!();
}

fn pickSession(sessions: &[SessionInfo]) -> crate::Result<Option<SessionInfo>> {
    let options: Vec<String> = sessions
        .iter()
        .map(|s| {
            let short_id = if s.id.len() > 15 {
                format!("{}...", &s.id[..15])
            } else {
                s.id.clone()
            };
            let label = if s.title.len() > 50 {
                format!("{}...", &s.title[..47])
            } else {
                s.title.clone()
            };
            format!("{}  │  {}", label, short_id)
        })
        .collect();

    let selection = Select::new("Select session to delete:", options.clone())
        .with_help_message("Enter to confirm, Arrow keys to navigate")
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    let idx = options.iter().position(|o| o == &selection).unwrap_or(0);
    Ok(Some(SessionInfo {
        id: sessions[idx].id.clone(),
        title: sessions[idx].title.clone(),
        updated: sessions[idx].updated.clone(),
    }))
}

fn pickMultipleSessions(sessions: &[SessionInfo]) -> crate::Result<Vec<SessionInfo>> {
    let options: Vec<String> = sessions
        .iter()
        .map(|s| {
            let short_id = if s.id.len() > 15 {
                format!("{}...", &s.id[..15])
            } else {
                s.id.clone()
            };
            let label = if s.title.len() > 50 {
                format!("{}...", &s.title[..47])
            } else {
                s.title.clone()
            };
            format!("{}  │  {}", label, short_id)
        })
        .collect();

    let selections = MultiSelect::new("Select sessions to delete:", options.clone())
        .with_help_message("Space to select, Enter to confirm, Arrow keys to navigate")
        .prompt()
        .map_err(|e| crate::Error::other(e.to_string()))?;

    let selected: Vec<SessionInfo> = selections
        .iter()
        .filter_map(|sel| {
            let idx = options.iter().position(|o| o == sel)?;
            Some(SessionInfo {
                id: sessions[idx].id.clone(),
                title: sessions[idx].title.clone(),
                updated: sessions[idx].updated.clone(),
            })
        })
        .collect();

    Ok(selected)
}

fn deleteSession(id: &str) -> crate::Result<()> {
    let output = Command::new("opencode")
        .args(["session", "delete", id])
        .output()
        .map_err(|e| crate::Error::other(format!("Failed to delete session: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::Error::other(format!(
            "{}",
            stderr.trim()
        )));
    }

    Ok(())
}
