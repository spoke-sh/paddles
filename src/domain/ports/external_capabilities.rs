use crate::domain::model::{
    ExternalCapabilityDescriptor, ExternalCapabilityInvocation, ExternalCapabilityResult,
};
use anyhow::Result;

pub trait ExternalCapabilityBroker: Send + Sync {
    fn descriptors(&self) -> Vec<ExternalCapabilityDescriptor>;

    fn invoke(&self, invocation: &ExternalCapabilityInvocation)
    -> Result<ExternalCapabilityResult>;

    fn descriptor(&self, capability_id: &str) -> Option<ExternalCapabilityDescriptor> {
        self.descriptors()
            .into_iter()
            .find(|descriptor| descriptor.id == capability_id)
    }
}
