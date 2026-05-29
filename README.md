# LOMA — Local LLM Optimizer & Manager Assistant

Loma is a modern command-line interface (CLI) tool written in Rust, designed to simplify and accelerate the setup and configuration of local AI coding assistants. By centralizing best practices in AI-driven development, Loma eliminates the friction associated with environment initialization, system rules enforcement, and workflow automation.

It is built on a fundamental but often neglected principle: even on developer machines equipped with 32, 64, or 128 GB of RAM, resource efficiency remains critical. The same principle applies to LLMs — just because an AI model possesses an enormous context window does not mean developers should inject excessive, unoptimized data into it. Loma encourages precise, strategic, and targeted context management. One of its primary objectives is to minimize token consumption by filtering out superfluous files and retaining only high-signal information, resulting in more reliable, faster, and highly cost-effective AI interactions.

> [!NOTE]  
> While Loma's initial implementation focuses on optimizing **Claude Code**, its core architecture is modular and extensible. It is designed to scale and support other AI coding assistants, command proxies, and local model controllers in the future.

---

## 📋 Table of Contents
- [✨ Key Features](#-key-features)
- [⚙️ Prerequisites](#️-prerequisites)
- [🚀 Installation](#-installation)
- [🔨 Build & Test](#-build--test)
- [📁 Project Structure](#-project-structure)
  - [Workspace Architecture Model](#workspace-architecture-model)
- [💻 CLI Commands](#-cli-commands)
- [🎮 Best Practices for Context Optimization](#-best-practices-for-context-optimization)
- [📦 Ref & ecosystem](#-ref--ecosystem)
- [📄 License](#-license)

---

## ✨ Key Features

* 🛠️ **Unified CLI & API Engine**: Offers a robust set of command-line tools for manual workflows, backed by an embedded high-performance Axum HTTP REST server for remote orchestration.
* 🩺 **Deep Environment Diagnostics (`health`)**: Automatically evaluates system prerequisites, global package managers, Node.js runtimes, network registry latency, and file permissions.
* 📦 **Automated Lifecycle Management**: Handles package installation, complete teardowns, clean reinstalls, and background updates seamlessly under the hood.
* 💾 **Interactive Backup & Restore**: Safeguards configurations, histories, and credential templates with structured compression and recovery utilities (operating relative to project root for Claude).
* ⚙️ **Smart Init & Settings Scaffolder**: Bootstraps project-native assistant folders, local `loma.env` configurations, and systemd service templates.
* ⚡ **Context & Token Minimizer**: Helps developers enforce token budgets using highly optimized, modular prompts and injection configurations.
* 🌀 **Multi-Stage Interactive Guideline Generator**: Generates/injects modular prompts interactively via a robust Rust TUI.
* 🔀 **Markdown-Aware Collision Management**: Intelligently resolves directory naming conflicts with options like Overwrite, Duplicate, or Merge (combining headers and bullet lists uniquely, validated structurally via `pulldown-cmark`).
* 🔗 **Internal Injection Registry**: Tracks and maintains historical prompt integrity with FNV-1a content hash tracking under `.loma/registry/injections.json`.
* 🔄 **Diagnostic Sync & Repair (`sync`)**: Compares on-disk configs, identifies duplicates, verifies settings JSON validity, and repairs mapping registries dynamically.

---

## ⚙️ Prerequisites

To run Loma and its managed AI assistants, ensure your environment meets the following conditions:
* **Operating System**: Linux (Fedora/DNF is natively integrated, but compatible with most major distributions).
* **Runtimes**: Node.js (version 18 or higher) and `npm` installed globally.
* **Network & Shell**: `curl` and `git` command-line tools must be available.

---

## 🚀 Installation

### Option A — Cargo Install (Recommended)
Compile and install Loma directly from your local Rust environment:
```bash
cargo install --path .
```

### Option B — Pre-compiled Binary
1. Download the latest release binary for your architecture from GitHub Releases.
2. Make the binary executable:
   ```bash
   chmod +x loma
   ```
3. Move the binary into your system `$PATH` (e.g., `/usr/local/bin/`):
   ```bash
   mv loma /usr/local/bin/
   ```

---

## 🔨 Build & Test

Manage Loma development using standard Cargo workflows:

```bash
cargo build --release                                         # Standard production build
cargo build --release --target x86_64-unknown-linux-musl      # Fully static musl binary

cargo test                  # Execute all unit and integration tests
cargo clippy -- -D warnings # Run strict linter checks
cargo fmt --check           # Verify code formatting matches style guidelines
```

---

## 📁 Project Structure

Below is the file tree of Loma's source code (`src/`), detailing the responsibility of each module:

```txt
src
├── api.rs          (<Axum HTTP API server for remote administration and automation endpoints>)
├── cli.rs          (<CLI definition using clap parser for subcommands like init, install, remove, etc.>)
├── commands        (<Folder containing individual subcommand implementations>)
│   ├── backup.rs   (<Logic for backup sub-command to back up configs and credentials>)
│   ├── gen.rs      (<Logic to generate configuration templates or systemd unit files>)
│   ├── gen_interactive.rs (<Multi-stage interactive TUI, markdown merging, and naming format rules>)
│   ├── health.rs   (<Environment diagnostics like check npm, dnf, curl, and network connectivity>)
│   ├── init.rs     (<Initializes local application files and scaffolds native assistant folders>)
│   ├── install.rs  (<Logic to install the AI coding assistant CLI via npm>)
│   ├── mod.rs      (<Rust module declarations and re-exports for the commands module>)
│   ├── optimize.rs (<Skeleton implementation for assistant configuration and optimization>)
│   ├── reinstall.rs(<Cleans and runs a fresh installation of the AI assistant>)
│   ├── remove.rs   (<Removes and cleans up all AI assistant installation and config directories>)
│   ├── restore.rs  (<Logic to restore configuration/history from backups>)
│   ├── status.rs   (<Checks current installation status, active processes, and versions>)
│   ├── sync.rs     (<Sync & Repair engine verifying integrity, hash duplicates, and mapping repairs>)
│   └── update.rs   (<Logic to check and perform upgrades of the installed AI assistant>)
├── config.rs       (<Application environment configuration management loading via dotenv>)
├── error.rs        (<Centralized error definitions and Result alias using thiserror/anyhow>)
├── json
│   └── inject.json (<JSON configuration or context/rules templates injected into AI assistants>)
├── lib.rs          (<Library root exposing public modules and constants>)
├── main.rs         (<Executable entry point parsing CLI inputs and running appropriate tasks>)
└── utils           (<General utility functions>)
    ├── banner.rs   (<Renders the startup ASCII text banner>)
    ├── const.rs    (<Central CLI/assistant constant configurations>)
    ├── display.rs  (<Provides beautiful, color-coded logging and terminal output utilities>)
    ├── env.rs      (<Environment detection helper functions>)
    ├── fs.rs       (<Advanced file system utilities for robust copy, validation, and permissions checking>)
    └── mod.rs      (<Rust module declarations for utilities>)
```

### Workspace Architecture Model

Loma separates internal engine maintenance from native developer workspace tools:

```
my-project/
├── CLAUDE.md                   # Minimal bootstrap file referencing native rules
├── .claude.json                # User credential, session history, and tokens
├── .claude/                    # Native Claude environment (Direct root access)
│   ├── settings.json           # General watchPatterns and assistant configs
│   ├── rules/                  # Modular rules (UPPERCASE_SNAKE_CASE_RULES.md)
│   ├── agents/                 # Custom agents (UPPERCASE_SNAKE_CASE_AGENTS.md)
│   ├── skills/                 # Extracted skills (UPPERCASE_SNAKE_CASE_SKILLS.md)
│   └── commands/               # CLI tools commands (UPPERCASE_SNAKE_CASE_COMMANDS.md)
└── .loma/                      # Internal Loma Workspace (Ignored in VCS)
    ├── loma.env                # Local app configuration
    ├── logs/                   # Isolated command and server logs
    ├── archives/               # Local compressed backups (relative format)
    └── registry/
        └── injections.json     # Injection history registry with FNV-1a hashing
```

---

## 💻 CLI Commands

Loma provides a full suite of commands to handle the lifecycle and optimization of your local AI assistant:

| Command | Description |
| :--- | :--- |
| `loma init [<assistant>]` | Bootstraps workspace-specific configurations (`.loma/loma.env` and scaffolds native `.claude/` directories). |
| `loma install [<assistant>]` | Installs the managed AI coding assistant package globally via the package manager. |
| `loma status [<assistant>]` | Displays the current installation status, active process tree, and installed versions. |
| `loma health [<assistant>]` | Runs diagnostic diagnostic routines verifying system prerequisites, permissions, and network connectivity. |
| `loma update [<assistant>]` | Upgrades the installed AI coding assistant package to the latest available release. |
| `loma backup [<assistant>]` | Starts an interactive wizard to compress and back up configurations relative to the target's natural directories. |
| `loma restore [<assistant>]` | Starts an interactive wizard to safely restore configuration backups directly back into place. |
| `loma reinstall [<assistant>]` | Purges the current assistant binary and configs, followed by a clean setup. |
| `loma remove [<assistant>]` | Completely uninstalls the AI assistant binary and removes all temporary and cache folders. |
| `loma optimize [<assistant>]` | Optimizes configurations for the targeted AI coding assistant (skeleton). |
| `loma gen [<assistant>]` | Triggers the Multi-Stage Interactive TUI for modular, token-saving files creation in `.claude/`. |
| `loma sync [<assistant>]` | Validates directory health, checks for duplicate hashes, and dynamically repairs `.loma/registry/injections.json`. |
| `loma api [--port <port>]` | Starts the embedded Axum HTTP server to expose control endpoints. |
| `loma run [--mode <mode>]` | Runs custom background logic or integration hooks. |
| `loma info [--verbose]` | Prints CLI binary metadata, compiled features, and repository coordinates. |

---

## 🎮 Best Practices for Context Optimization

To maximize the efficiency of your AI coding assistant and save token costs, apply these strategic practices:

1. **Precision Context**: Only feed the files you are actively working on. Avoid passing entire large directories if only a single module needs to be edited.
2. **Strict Rule Injections**: Use `.claude/rules/` to restrict verbose responses, prevent the AI from refactoring adjacent unrelated code, and force concise, surgical updates.
3. **Environment Hygiene**: Periodically run `loma health` to ensure your system runtimes are optimal and network latency to registries is minimized.
4. **History Rotation**: Large prompt histories can accumulate bloat over time. Make frequent use of `loma backup` and occasionally wipe log caches using `loma reinstall` to start fresh.

---

## 📦 Ref & ecosystem

Here is a curated list of high-quality tools, proxies, and libraries within the AI-driven development ecosystem designed to optimize context windows, token consumption, and agent workflows:

| Repository | Description | Key Focus / Benefit |
| :--- | :--- | :--- |
| [🔗 code-review-graph](https://github.com/tirth8205/code-review-graph) | Local-first code intelligence graph for MCP and CLI. Builds a persistent map of your codebase. | Reduces context on reviews & large workflows. |
| [🔗 claude-context](https://github.com/zilliztech/claude-context) | Code search Model Context Protocol (MCP) server designed for Claude Code. | Makes the entire codebase searchable context. |
| [🔗 context-mode](https://github.com/mksglu/context-mode) | Context window optimization server for AI coding agents. | Sandboxes tool output, leading to 98% reduction. |
| [🔗 caveman](https://github.com/juliusbrussee/caveman) | A Claude Code custom skill that forces the model to speak like a caveman. | Cuts 65% of output token consumption. |
| [🔗 ccusage](https://github.com/ryoppippi/ccusage) | Diagnostic token analyzer for local coding agent logs. | Estimates token consumption and real costs. |
| [🔗 token-optimizer](https://github.com/alexgreensh/token-optimizer) | Diagnostic tool designed to identify and fix "ghost tokens" inside contexts. | Prevents model context decay and confusion. |
| [🔗 rtk](https://github.com/rtk-ai/rtk) | High-performance CLI proxy reducing LLM token consumption. | 60-90% token reduction on standard dev commands. |
| [🔗 free-claude-code](https://github.com/Alishahryar1/free-claude-code) | Alternative runner proxy providing voice and custom integrations. | Use Claude Code for free, with voice support. |
| [🔗 claude-task-master](https://github.com/eyaltoledano/claude-task-master) | AI-powered task orchestration framework for Cursor, Windsurf, Roo, etc. | Droppable agent workflow controller. |
| [🔗 taskmaster-cli](https://github.com/RajaRakoto/taskmaster-cli) | Interactive command-line workflow manager orchestrating multiple agents. | Seamless planning and execution in terminal. |

> 💡 **Bonus Tip:** Check out [GitHub Token Savers](https://vishnuai.in/github-token-savers) for more ways to optimize your AI-driven coding budget!

---

## 📄 License

LOMA is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.