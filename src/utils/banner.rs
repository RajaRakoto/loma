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
