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

    Ok(())
}
