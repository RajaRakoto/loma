use crate::providers;

pub fn runRemove(assistant: &str) -> crate::Result<()> {
    let provider = providers::getProvider(assistant)?;
    provider.remove()
}
