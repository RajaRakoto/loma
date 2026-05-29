use crate::providers;

pub fn runUpdate(assistant: &str) -> crate::Result<()> {
    let provider = providers::getProvider(assistant)?;
    provider.update()
}
