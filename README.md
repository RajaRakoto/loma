# LOMA — LLM Optimizer & Manager Assistant

Loma manages, configures, and optimizes AI coding assistants. It centralizes prompt engineering standards and configuration, reducing token consumption by **60–90%** while keeping your development loops fast and cost-effective.

> **Supported assistants**: [OpenCode](https://opencode.ai) (recommended) · [Claude Code](https://claude.ai)

---

## Quick Start

```bash
# Install
cargo install --path .

# Initialize your assistant (start here)
loma init opencode        # scaffolds AGENTS.md + global ~/.config/opencode/
loma doctor               # check environment health
loma optimize opencode    # apply optimized configs + ignore patterns
```

---

## OpenCode vs Claude Code

| Area | OpenCode (`opencode`) | Claude Code (`claude`) |
| :--- | :--- | :--- |
| **Config model** | Global-first — `~/.config/opencode/` | Project-local — `.claude/` |
| **Default model** | `deepseek/deepseek-v4-flash` | Anthropic Claude Sonnet |
| **Cost** | Substantially lower token cost | Higher per-token cost |
| **Sub-agents** | Native (agents in `~/.config/opencode/agents/`) | Not supported |
| **Plan mode** | Built-in (`/plan`, `/compact`) | Manual |

Loma applies the same environment-tuning pipeline to both — diagnostics, compaction thresholds, model routing, ignore patterns, and interactive guideline generation.

---

## Commands

### Lifecycle
| Command | Description |
| :--- | :--- |
| `loma init <assistant>` | Scaffold project config files (AGENTS.md / CLAUDE.md, registries) |
| `loma install <assistant>` | Install the assistant binary globally |
| `loma remove <assistant>` | Remove binary, caches, and config directories |
| `loma reinstall <assistant>` | Purge + fresh install |
| `loma update <assistant>` | Upgrade to the latest version |

### Configuration
| Command | Description |
| :--- | :--- |
| `loma optimize <assistant>` | Apply optimized JSON configs, env vars, ignore patterns |
| `loma gen <assistant>` | Interactive TUI to inject domain guidelines (Git, Docker, Dev…) |
| `loma skills <assistant>` | Manage on-demand modular sub-agent skills |
| `loma sync <assistant>` | Validate and repair config registries and native structures |

### Diagnostics
| Command | Description |
| :--- | :--- |
| `loma status <assistant>` | Show version, binary path, and metrics |
| `loma doctor` | Check runtime, tools, and config health |
| `loma tips <assistant>` | Token-saving and prompt-caching strategies |

### Data & Recovery
| Command | Description |
| :--- | :--- |
| `loma backup <assistant>` | Archive configs, AGENTS.md/CLAUDE.md, and global settings |
| `loma restore <assistant>` | Restore from a previous backup |
| `loma usage` | Track token consumption and costs |

### Reference
| Command | Description |
| :--- | :--- |
| `loma tutorial [tool]` | Step-by-step guides for RTK, Caveman, Graphify, and others |
| `loma info` | Print build and environment details |

---

## Environment Configuration

Loma loads workspace parameters from `.loma/loma.env`. Settings follow `LOMA_<ASSISTANT>_<KEY>` naming to avoid collisions:

```env
# OpenCode (global — ~/.config/opencode/)
LOMA_OPENCODE_MODEL=deepseek/deepseek-v4-flash
LOMA_OPENCODE_COMPACT_THRESHOLD=3000

# Claude Code (project-local — .claude/)
LOMA_CLAUDE_MODEL=claude-sonnet-4-20250514
LOMA_CLAUDE_EFFORT_LEVEL=high
```

For API keys, export them in your shell profile rather than storing them in config files:

```bash
export ANTHROPIC_AUTH_TOKEN="sk-ant-..."
# or for OpenCode / generic providers:
export OPENAI_API_KEY="sk-..."
```

---

## Integrated Tools

Loma coordinates external utilities to minimize context bloat:

| Tool | Purpose | Savings |
| :--- | :--- | :--- |
| **RTK** | Compress shell output (git, tests, lint) | −70% to −92% |
| **Caveman** | Enforce telegraphic model responses | −65% |
| **Graphify** | Index codebase as a queryable knowledge graph | Architectural |
| **Code Review Graph** | Limit review context to changed files | 40×–500× |
| **ccusage** | Local token/cost tracking | N/A (metrics) |

```bash
loma tutorial <tool>   # view setup guide
```

---

## License

MIT — see [LICENSE](LICENSE).
