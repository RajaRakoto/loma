use crate::providers;

pub fn runInstall(assistant: &str) -> crate::Result<()> {
    let provider = providers::getProvider(assistant)?;
    provider.install(false)
}
