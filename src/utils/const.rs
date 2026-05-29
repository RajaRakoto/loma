//! CLI constants configuration

pub const CLAUDE_CONFIG_DIRS: &[&str] = &[".claude"];
pub const CLAUDE_CONFIG_FILES: &[&str] = &[".claude.json"];
pub const CLAUDE_DATA_DIRS: &[&str] = &[
    ".local/share/claude",
    ".cache/claude",
    ".cache/@anthropic-ai",
];
pub const CLAUDE_DNF_REPO_FILES: &[&str] = &[
    "/etc/yum.repos.d/claude-code.repo",
    "/etc/yum.repos.d/anthropic-claude.repo",
];
pub const CLAUDE_BINARY_PATHS: &[&str] = &[
    ".local/bin/claude",
    "/usr/local/bin/claude",
    "/usr/bin/claude",
    ".npm-global/bin/claude",
    ".bun/bin/claude",
];
