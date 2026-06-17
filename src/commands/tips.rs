use crate::utils::display;

pub fn runTips(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Tips & Guidelines ({})", assistant));

    display::step("Prompt Cache & MCP Guidelines");
    display::info("Prompt caching is extremely powerful for long-running sessions, reducing costs by up to 90%.");
    println!("\nRules to protect your prompt cache hit rate:");
    println!("  • Lock Model: Never change model (via /model) mid-session. It invalidates the cache.");
    println!("  • Lock Tools: Do not add or remove MCP servers during a session.");
    println!("  • Clean Start: Begin major tasks with a plan. Correcting plans is cheaper than undoing steps.");
    println!("  • Separate Concerns: Separate frontend, backend, and infra tasks into different domains/sessions.");
    println!("  • Clear context: Run /clear between unrelated tasks to avoid history contamination.");
    println!("  • Manual Compaction: Use /compact when the context grows to ~50% to clear out transient garbage.");
    println!("  • Skills Separation: Move rare deployment steps or migrations out of CLAUDE.md into specific files in .claude/skills/.");

    display::step("General Token-Saving Rules");
    println!("  • Use precise prompts containing: the file, exact objective, and constraints. Do not ask vague questions.");
    println!("  • Delegate mechanical tasks or research to subagents using cheaper models.");

    display::step("Model Routing & Auto-Compaction Configuration");
    println!("  • Route subagents to a cheaper model (e.g. claude-haiku-4-5) to save tokens.");
    println!("  • Set effortLevel to medium to reduce silent thinking tokens.");
    println!("  • Limit max output tokens to 4096 to prevent verbose explanations.");
    println!("  • Configure auto-compaction aggressively (COMPACT_ON_DEMAND: true, 50% threshold) to maintain a lean context.");

    display::step("Third-Party Optimization Tools");
    println!("  • RTK (Rust Token Kill): Compresses shell output (git, tsc, cargo, tests, docker) by 70% to 92%.");
    println!("  • Caveman: Enforces telegraphic answers to minimize output tokens.");
    println!("  • ccusage: Compiles local token consumption reports.");
    println!("  • Graphify: Builds codebase knowledge graphs, avoiding massive file reads.");
    println!("  • Code Review Graph: Maps dependencies to target only modified files.");

    Ok(())
}
