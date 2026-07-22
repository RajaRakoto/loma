//! Provider trait and dynamic resolver for AI assistants.

pub trait AssistantProvider {
    fn install(&self, force: bool) -> crate::Result<()>;
    fn reinstall(&self) -> crate::Result<()>;
    fn remove(&self) -> crate::Result<()>;
    fn update(&self) -> crate::Result<()>;
    fn status(&self) -> crate::Result<()>;
}

pub fn getProvider(assistant: &str) -> crate::Result<Box<dyn AssistantProvider>> {
    match assistant.to_lowercase().as_str() {
        "claude" => Ok(Box::new(claude::ClaudeProvider::new())),
        "opencode" => Ok(Box::new(opencode::OpenCodeProvider::new())),
        _ => Err(crate::Error::validation(format!(
            "Unknown or unsupported assistant: {}",
            assistant
        ))),
    }
}

pub mod claude;
pub mod opencode;
