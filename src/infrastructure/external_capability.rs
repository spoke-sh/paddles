use crate::domain::model::{
    ExternalCapabilityDescriptor, ExternalCapabilityInvocation, ExternalCapabilityResult,
    default_external_capability_descriptors,
};
use crate::domain::ports::ExternalCapabilityBroker;
use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct NoopExternalCapabilityBroker {
    descriptors: Vec<ExternalCapabilityDescriptor>,
}

impl Default for NoopExternalCapabilityBroker {
    fn default() -> Self {
        Self::new(default_external_capability_descriptors())
    }
}

impl NoopExternalCapabilityBroker {
    pub fn new(descriptors: Vec<ExternalCapabilityDescriptor>) -> Self {
        Self { descriptors }
    }
}

impl ExternalCapabilityBroker for NoopExternalCapabilityBroker {
    fn descriptors(&self) -> Vec<ExternalCapabilityDescriptor> {
        self.descriptors.clone()
    }

    fn invoke(
        &self,
        invocation: &ExternalCapabilityInvocation,
    ) -> Result<ExternalCapabilityResult> {
        let descriptor = self
            .descriptor(&invocation.capability_id)
            .ok_or_else(|| anyhow!("unknown external capability `{}`", invocation.capability_id))?;
        Ok(ExternalCapabilityResult::unavailable(
            descriptor.clone(),
            invocation.clone(),
            format!(
                "{} is currently {} in this runtime",
                descriptor.label,
                descriptor.availability.label()
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::NoopExternalCapabilityBroker;
    use crate::domain::model::{
        ExternalCapabilityAvailability, ExternalCapabilityInvocation,
        ExternalCapabilityResultStatus,
    };
    use crate::domain::ports::ExternalCapabilityBroker;
    use serde_json::json;

    #[test]
    fn noop_broker_publishes_the_default_catalog() {
        let broker = NoopExternalCapabilityBroker::default();
        let descriptors = broker.descriptors();

        assert_eq!(descriptors.len(), 3);
        assert!(descriptors.iter().all(|descriptor| {
            descriptor.availability == ExternalCapabilityAvailability::Unavailable
        }));
    }

    #[test]
    fn noop_broker_returns_explicit_unavailable_results_for_known_capabilities() {
        let broker = NoopExternalCapabilityBroker::default();
        let result = broker
            .invoke(&ExternalCapabilityInvocation::new(
                "web.search",
                "look up current docs",
                json!({ "query": "paddles" }),
            ))
            .expect("known capability should produce a typed result");

        assert_eq!(result.status, ExternalCapabilityResultStatus::Unavailable);
        assert!(result.detail.contains("currently unavailable"));
    }
}
