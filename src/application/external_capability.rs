use crate::domain::model::{
    ExternalCapabilityCatalog, ExternalCapabilityCatalogConfig, ExternalCapabilityDescriptor,
    ExternalCapabilityInvocation, ExternalCapabilityResult,
};
use crate::domain::ports::ExternalCapabilityBroker;
use anyhow::{Result, anyhow};

#[derive(Clone, Debug)]
pub struct ExternalCapabilityBrokerRegistry {
    catalog: ExternalCapabilityCatalog,
}

impl Default for ExternalCapabilityBrokerRegistry {
    fn default() -> Self {
        Self::from_catalog(ExternalCapabilityCatalog::default())
    }
}

impl ExternalCapabilityBrokerRegistry {
    pub fn from_catalog(catalog: ExternalCapabilityCatalog) -> Self {
        Self { catalog }
    }

    pub fn from_local_configuration(config: ExternalCapabilityCatalogConfig) -> Self {
        Self::from_catalog(ExternalCapabilityCatalog::from_local_configuration(&config))
    }

    pub fn catalog(&self) -> &ExternalCapabilityCatalog {
        &self.catalog
    }
}

impl ExternalCapabilityBroker for ExternalCapabilityBrokerRegistry {
    fn descriptors(&self) -> Vec<ExternalCapabilityDescriptor> {
        self.catalog.descriptors()
    }

    fn invoke(
        &self,
        invocation: &ExternalCapabilityInvocation,
    ) -> Result<ExternalCapabilityResult> {
        let descriptor = self
            .descriptor(&invocation.capability_id)
            .ok_or_else(|| anyhow!("unknown external capability `{}`", invocation.capability_id))?;
        let detail = if descriptor.availability.is_usable() {
            format!(
                "{} is declared available, but no local executor is registered in this runtime",
                descriptor.label
            )
        } else {
            format!(
                "{} is currently {} in this runtime",
                descriptor.label,
                descriptor.availability.label()
            )
        };
        Ok(ExternalCapabilityResult::unavailable(
            descriptor,
            invocation.clone(),
            detail,
        ))
    }

    fn descriptor(&self, capability_id: &str) -> Option<ExternalCapabilityDescriptor> {
        self.catalog.descriptor(capability_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::model::{ExternalCapabilityAvailability, ExternalCapabilityCatalogConfig};
    use crate::domain::ports::ExternalCapabilityBroker;

    #[test]
    fn external_capability_broker_registry_exposes_configured_catalog_through_port() {
        let registry = super::ExternalCapabilityBrokerRegistry::from_local_configuration(
            ExternalCapabilityCatalogConfig::default().enable("web.search"),
        );
        let broker: &dyn ExternalCapabilityBroker = &registry;

        let web = broker
            .descriptor("web.search")
            .expect("configured web capability");
        let mcp = broker
            .descriptor("mcp.tool")
            .expect("default mcp capability");

        assert_eq!(web.availability, ExternalCapabilityAvailability::Available);
        assert_eq!(
            mcp.availability,
            ExternalCapabilityAvailability::Unavailable
        );
    }

    #[test]
    fn external_capability_default_posture_keeps_catalog_unavailable_until_enabled() {
        let default_registry = super::ExternalCapabilityBrokerRegistry::default();
        assert!(default_registry.descriptors().iter().all(
            |descriptor| descriptor.availability == ExternalCapabilityAvailability::Unavailable
        ));

        let configured_registry = super::ExternalCapabilityBrokerRegistry::from_local_configuration(
            ExternalCapabilityCatalogConfig::default().enable("web.search"),
        );
        let descriptors = configured_registry.descriptors();

        assert_eq!(
            descriptors
                .iter()
                .find(|descriptor| descriptor.id == "web.search")
                .map(|descriptor| descriptor.availability),
            Some(ExternalCapabilityAvailability::Available)
        );
        assert_eq!(
            descriptors
                .iter()
                .filter(|descriptor| {
                    descriptor.availability == ExternalCapabilityAvailability::Available
                })
                .count(),
            1
        );
    }
}
