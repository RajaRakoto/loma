pub fn showBanner() {
    println!("\x1b[38;5;93m    ██╗      ██████╗ ███╗   ███╗ █████╗ \x1b[0m");
    println!("\x1b[38;5;129m    ██║     ██╔═══██╗████╗ ████║██╔══██╗\x1b[0m");
    println!("\x1b[38;5;165m    ██║     ██║   ██║██╔████╔██║███████║\x1b[0m");
    println!("\x1b[38;5;201m    ██║     ██║   ██║██║╚██╔╝██║██╔══██║\x1b[0m");
    println!("\x1b[38;5;164m    ███████╗╚██████╔╝██║ ╚═╝ ██║██║  ██║\x1b[0m");
    println!("\x1b[38;5;99m    ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝\x1b[0m");
    println!();
    println!(
        "\x1b[1;35m  ✦ Loma: LLM Optimizer & Manager ✦ \x1b[0;32mv{}\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "\x1b[1;30m  Author: \x1b[0;36m{}\x1b[0m",
        env!("CARGO_PKG_AUTHORS")
    );
    println!("\x1b[2;37m  ───────────────────────────────────────────────\x1b[0m");
}

pub fn showHelp() {
    println!("\x1b[1mUsage:\x1b[0m loma [OPTIONS] <COMMAND>");
    println!();
    println!("\x1b[1mOptions:\x1b[0m");
    println!("  \x1b[32m-h, --help\x1b[0m       Print help information");
    println!("  \x1b[32m-v, --version\x1b[0m    Print version information");
    println!();
    println!("\x1b[1mCommands:\x1b[0m");
    println!();
    println!("  \x1b[35;1mSetup & Initialization\x1b[0m");
    println!("    \x1b[36minit\x1b[0m          Initialize configuration files for loma");
    println!("    \x1b[36minstall\x1b[0m       Install an AI assistant");
    println!("    \x1b[36mremove\x1b[0m        Completely remove an AI assistant and all associated files");
    println!("    \x1b[36mreinstall\x1b[0m     Remove then cleanly reinstall an AI assistant");
    println!();
    println!("  \x1b[35;1mManagement & Optimization\x1b[0m");
    println!("    \x1b[36mupdate\x1b[0m        Update an AI assistant");
    println!("    \x1b[36moptimize\x1b[0m      Optimize configuration for an AI assistant");
    println!("    \x1b[36mgen\x1b[0m           Generate guidelines/conventions for an assistant");
    println!();
    println!("  \x1b[35;1mMaintenance & Health\x1b[0m");
    println!("    \x1b[36mstatus\x1b[0m        Show current status of an AI assistant");
    println!("    \x1b[36mhealth\x1b[0m        Perform diagnostic health checks on the assistant's environment");
    println!("    \x1b[36mbackup\x1b[0m        Back up AI assistant configuration");
    println!("    \x1b[36mrestore\x1b[0m       Restore a previous backup of an AI assistant");
    println!();
    println!("  \x1b[35;1mGeneral / Metadata\x1b[0m");
    println!("    \x1b[36minfo\x1b[0m          Print application information");
    println!();
}

