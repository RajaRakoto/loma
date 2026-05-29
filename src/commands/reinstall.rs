use crate::providers;

pub fn runReinstall(assistant: &str) -> crate::Result<()> {
    let provider = providers::getProvider(assistant)?;
    provider.reinstall()
}
