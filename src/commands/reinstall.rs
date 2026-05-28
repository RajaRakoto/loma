use crate::utils::display;
use std::thread;
use std::time::Duration;

pub fn runReinstall() -> crate::Result<()> {
    display::title("Complete Reinstall of Claude Code");
    display::info("This will first remove Claude Code entirely, then perform a clean install.");
    println!();

    if !display::confirm("Proceed with full reinstall?") {
        display::info("Reinstall cancelled.");
        return Ok(());
    }

    crate::commands::remove::runRemove()?;
    println!();
    display::info("Waiting 2 seconds before reinstalling...");
    thread::sleep(Duration::from_secs(2));

    crate::commands::install::runInstall()?;

    Ok(())
}
