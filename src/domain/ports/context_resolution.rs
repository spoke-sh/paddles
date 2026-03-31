use async_trait::async_trait;
use paddles_conversation::ContextLocator;
use std::fmt::Debug;

/// Port for resolving addressable context locators to full artifact content.
#[async_trait]
pub trait ContextResolver: Send + Sync + Debug {
    /// Resolve a locator to its full artifact content.
    async fn resolve(&self, locator: &ContextLocator) -> anyhow::Result<String>;
}
