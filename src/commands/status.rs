use crate::providers;

pub fn runStatus(assistant: &str) -> crate::Result<()> {
    let provider = providers::getProvider(assistant)?;
    provider.status()
}
