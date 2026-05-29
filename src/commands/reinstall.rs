use crate::utils::display;
use std::thread;
use std::time::Duration;

pub fn runReinstall(assistant: &str) -> crate::Result<()> {
    display::title(&format!("Complete Reinstall of {}", assistant));
    display::info(&format!("This will first remove {} entirely, then perform a clean install.", assistant));
    println!();

    if !display::confirm(&format!("Proceed with full reinstall of {}?", assistant)) {
        display::info("Reinstall cancelled.");
        return Ok(());
    }

    crate::commands::remove::runRemove(assistant)?;
    println!();
    display::info("Waiting 2 seconds before reinstalling...");
    thread::sleep(Duration::from_secs(2));

    crate::commands::install::runInstall(assistant)?;

    Ok(())
}
