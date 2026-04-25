use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliberationState {
    provider_name: String,
    runtime_model_id: String,
    payload: Value,
}

impl DeliberationState {
    pub fn new(
        provider_name: impl Into<String>,
        runtime_model_id: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            provider_name: provider_name.into(),
            runtime_model_id: runtime_model_id.into(),
            payload,
        }
    }

    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    pub fn runtime_model_id(&self) -> &str {
        &self.runtime_model_id
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }
}
