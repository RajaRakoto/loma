//! CLI constants configuration

pub const CLAUDE_CONFIG_DIRS: &[&str] = &[".claude"];
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

pub const OPENCODE_CONFIG_DIRS: &[&str] = &[".opencode"];
pub const OPENCODE_GLOBAL_CONFIG_DIR: &str = ".config/opencode";
pub const OPENCODE_BINARY_PATHS: &[&str] = &[
    ".local/bin/opencode",
    "/usr/local/bin/opencode",
    "/usr/bin/opencode",
];
pub const OPENCODE_DATA_DIRS: &[&str] = &[
    ".local/share/opencode",
    ".cache/opencode",
];
