use crate::utils::display;

pub fn runTips(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Tips & Guidelines ({})", assistant));

    let assistant_lower = assistant.to_lowercase();

    if assistant_lower == "opencode" {
        display::step("OpenCode — Plan Mode Discipline");
        println!("  • Start every non-trivial task in Plan mode (read-only).");
        println!("  • Describe your understanding, validate the approach BEFORE touching code.");
        println!("  • Switch to Build mode only after the plan is approved.");
        println!("  • Plan mode avoids costly back-and-forth corrections.");

        display::step("OpenCode — Compact Regularly");
        println!("  • Monitor the token counter in the TUI.");
        println!("  • Run /compact when context is ~50% full.");
        println!("  • Compact regularly on long sessions — do NOT wait for saturation.");
        println!("  • Each file read and tool output consumes tokens — be intentional.");

        display::step("OpenCode — MCP on Demand");
        println!("  • Activate only the MCP servers needed for the current task.");
        println!("  • Deactivate them immediately after the task.");
        println!("  • An unused but active MCP server wastes tokens on EVERY message.");

        display::step("OpenCode — Sub-Agent Strategy");
        println!("  • Delegate mechanical tasks to sub-agents (clean context, no pollution).");
        println!("  • Define reviewer sub-agents with read-only access.");
        println!("  • Sub-agents keep the main agent lean and focused.");

        display::step("OpenCode — Include/Exclude Patterns");
        println!("  • Exclude node_modules/, target/, dist/, *.lock files.");
        println!("  • Exclude logs, caches, binary assets, build artifacts.");
        println!("  • Every unnecessary file = wasted tokens.");

        display::step("OpenCode — One Session = One Task");
        println!("  • Open a new session per distinct task or topic.");
        println!("  • Mixed-topic long sessions = bloated context = rising costs.");
        println!("  • Close and reopen when the subject changes.");

        display::step("OpenCode — Custom Commands");
        println!("  • Define reusable prompts in opencode.json (custom commands).");
        println!("  • Example: a 'review' command that checks security, perf, errors.");
        println!("  • Custom commands replace long repetitive prompts with short names.");
    }

    display::step("Prompt Cache & MCP Guidelines");
    display::info("Prompt caching is extremely powerful for long-running sessions, reducing costs by up to 90%.");
    println!("\nRules to protect your prompt cache hit rate:");
    println!(
        "  • Lock Model: Never change model (via /model) mid-session. It invalidates the cache."
    );
    println!("  • Lock Tools: Do not add or remove MCP servers during a session.");
    println!("  • Clean Start: Begin major tasks with a plan. Correcting plans is cheaper than undoing steps.");
    println!("  • Separate Concerns: Separate frontend, backend, and infra tasks into different domains/sessions.");
    println!(
        "  • Clear context: Run /clear between unrelated tasks to avoid history contamination."
    );
    println!("  • Manual Compaction: Use /compact when the context grows to ~50% to clear out transient garbage.");
    println!("  • Skills Separation: Move rare deployment steps or migrations out of guidelines files into specific files in skills/agents directories.");

    display::step("General Token-Saving Rules");
    println!("  • Use precise prompts containing: the file, exact objective, and constraints. Do not ask vague questions.");
    println!("  • Delegate mechanical tasks or research to subagents using cheaper models.");

    display::step("Model Routing & Auto-Compaction Configuration");
    println!("  • Route subagents to a cheaper model to save tokens.");
    println!("  • Set effort level to medium to reduce silent thinking tokens.");
    println!("  • Limit max output tokens to 4096 to prevent verbose explanations.");
    println!("  • Configure auto-compaction aggressively to maintain a lean context.");

    display::step("Third-Party Optimization Tools");
    println!("  • RTK (Rust Token Kill): Compresses shell output (git, tsc, cargo, tests, docker) by 70% to 92%.");
    println!("  • Caveman: Enforces telegraphic answers to minimize output tokens.");
    println!("  • ccusage: Compiles local token consumption reports.");
    println!("  • Graphify: Builds codebase knowledge graphs, avoiding massive file reads.");
    println!("  • Code Review Graph: Maps dependencies to target only modified files.");

    Ok(())
}
