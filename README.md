# LOMA — LLM Optimizer & Manager Assistant

Loma is a modern command-line interface (CLI) tool written in Rust, designed to manage, configure, and optimize AI coding assistants (like Claude Code). 

By centralizing configuration management and prompt engineering standards, Loma reduces token consumption by **60% to 90%**, keeping your development loops fast, cost-effective, and precise.

> [!IMPORTANT]
> **Why Optimize Context?** Huge context windows do not mean you should flood the model with raw files. Minimizing token overhead yields faster response times, avoids context drifting, and saves massive API costs.

---

## 🚀 Quick Start (Beginner's Guide)

Follow these four simple steps to get Loma running and optimize your AI assistant:

### Step 1: Install Loma
Compile and install Loma directly into your local path:
```bash
cargo install --path .
```

### Step 2: Initialize your Assistant
Bootstrap the required configuration directories and local profiles:
```bash
loma init claude
```
This scaffolds `.claude/` directories, a default `CLAUDE.md` context file, and creates the internal registry.

### Step 3: Run Environment Diagnostics
Ensure your environment has all necessary global runtimes, platform variables, and package managers configured:
```bash
loma doctor
```

### Step 4: Optimize Configurations
Run the interactive optimizer to merge recommended settings (compaction thresholds, model routing) and create ignore patterns:
```bash
loma optimize claude
```

---

## 💻 Core Commands

Loma organizes tasks into distinct stages of configuration and maintenance:

### Setup & Lifecycle Management
* `loma init <assistant>` — Bootstraps workspace-specific structures and configurations.
* `loma install <assistant>` — Installs the target AI assistant globally using the system's package manager.
* `loma remove <assistant>` — Interactively cleans up binaries, local caches, and configuration directories.
* `loma reinstall <assistant>` — Purges current settings and executes a fresh installation.
* `loma update <assistant>` — Upgrades the AI coding assistant package to the latest version.

### Prompt & Context Engineering
* `loma optimize <assistant>` — Maps optimized JSON configurations and updates ignore patterns (`.claudeignore`).
* `loma gen <assistant>` — Multi-stage interactive TUI to select, group, and inject domain-specific guidelines (Git, Docker, Dev) into target directories.
* `loma skills <assistant>` — Manage custom, on-demand assistant skills (modular templates) inside the workspace.
* `loma sync <assistant> (beta)` — Validates workspace directory integrity, computes hashes, and repairs configuration registries.

### Diagnostics & Reference
* `loma status <assistant> (beta)` — Displays current version, binary path, and active process metrics.
* `loma doctor` — Evaluates global Node.js runtime, curl/git access, and registry latency.
* `loma tutorial [tool]` — Step-by-step setup tutorials for third-party optimization tools (RTK, Caveman, Graphify, etc.).
* `loma tips <assistant>` — Quick summary of prompt-caching rules and token-saving strategies.

---

## ⚙️ Environment Configuration

Loma supports isolated workspace parameters loaded from `.loma/loma.env`.

### Security Best Practices
Do not store sensitive credentials in configuration files. Instead, export them globally in your shell profile (`.bashrc`, `.zshrc`, or `config.fish`):
```bash
export ANTHROPIC_BASE_URL="https://api.anthropic.com"
export ANTHROPIC_AUTH_TOKEN="your-api-key-here"
```

### Prefixing
All Claude-specific optimization settings in `loma.env` are prefixed with `LOMA_CLAUDE_` (e.g. `LOMA_CLAUDE_MODEL`, `LOMA_CLAUDE_EFFORT_LEVEL`) to avoid conflicts when managing multiple assistants.

---

## 🛠️ Integrated Third-Party Tools

Loma configures and coordinates the following external utility tools to minimize context bloat:

| Tool | Action | Typical Savings |
| :--- | :--- | :--- |
| **RTK** | Compresses shell command output (git, tests, lint). | **−70% to −92%** |
| **ccusage** | Local token consumption and cost tracker. | N/A (metrics only) |
| **Caveman** | Enforces compact, telegraphic responses from the model. | **−65%** |
| **Code Review Graph** | Limits review context to modified files using AST dependencies. | **40x to 500x** |
| **Graphify** | Indexes the codebase as a queryable knowledge graph. | Massive (architecture) |

Access step-by-step documentation for these tools at any time using:
```bash
loma tutorial <tool_name>
```

---

## 📄 License

Loma is open-source software licensed under the [MIT License](LICENSE).