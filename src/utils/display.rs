use std::io::{self, Write};
use std::fs::OpenOptions;
use chrono::Local;

/// Log message to `claude-manager.log` in the current directory.
pub fn logToFile(level: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("claude-manager.log")
    {
        let _ = writeln!(file, "{} [{}] {}", timestamp, level, message);
    }
}

pub fn info(msg: &str) {
    println!("\x1b[0;36m➤\x1b[0m  \x1b[1m{}\x1b[0m", msg);
    logToFile("INFO", msg);
}

pub fn success(msg: &str) {
    println!("\x1b[0;32m✔\x1b[0m   \x1b[0;32m\x1b[1m{}\x1b[0m", msg);
    logToFile("SUCCESS", msg);
}

pub fn warn(msg: &str) {
    println!("\x1b[1;33m⚠\x1b[0m  \x1b[1;33m{}\x1b[0m", msg);
    logToFile("WARN", msg);
}

pub fn error(msg: &str) {
    eprintln!("\x1b[0;31m✖\x1b[0m  \x1b[0;31m\x1b[1m{}\x1b[0m", msg);
    logToFile("ERROR", msg);
}

pub fn step(msg: &str) {
    println!("\n\x1b[0;34m→\x1b[0m \x1b[1m\x1b[0;36m{}\x1b[0m", msg);
    logToFile("STEP", msg);
}

pub fn title(msg: &str) {
    println!("\n\x1b[0;35m\x1b[1m══════════════════════════════════════\x1b[0m");
    println!("\x1b[0;35m\x1b[1m  {}\x1b[0m", msg);
    println!("\x1b[0;35m\x1b[1m══════════════════════════════════════\x1b[0m\n");
    logToFile("TITLE", msg);
}

pub fn divider() {
    println!("\x1b[2m──────────────────────────────────────\x1b[0m");
}

pub fn confirm(prompt: &str) -> bool {
    print!("\x1b[1;33m\x1b[1m  {} [Y/n]: \x1b[0m", prompt);
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    let trimmed = input.trim().to_lowercase();
    trimmed.is_empty() || trimmed == "y" || trimmed == "yes"
}