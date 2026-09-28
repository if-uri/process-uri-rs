use crate::uri::{ProcessUri, ProcessUriError};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ActionDefinition {
    pub uri_pattern: String,
    pub description: String,
    pub required_params: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ProcessUriRegistry {
    actions: BTreeMap<String, ActionDefinition>,
}

impl ProcessUriRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, uri_pattern: &str, description: &str, required_params: Vec<&str>) {
        self.actions.insert(
            uri_pattern.to_string(),
            ActionDefinition {
                uri_pattern: uri_pattern.to_string(),
                description: description.to_string(),
                required_params: required_params.into_iter().map(|s| s.to_string()).collect(),
            },
        );
    }

    pub fn get(&self, uri_pattern: &str) -> Option<&ActionDefinition> {
        self.actions.get(uri_pattern)
    }

    pub fn actions(&self) -> &BTreeMap<String, ActionDefinition> {
        &self.actions
    }

    pub fn validate(&self, uri: &ProcessUri, payload: &Value) -> Result<(), ProcessUriError> {
        let action = self
            .get(&uri.canonical_action())
            .ok_or_else(|| ProcessUriError::MissingScheme(format!("Action '{}' not registered", uri.canonical_action())))?;

        for req in &action.required_params {
            let in_query = uri.query_params.contains_key(req);
            let in_payload = payload.get(req).is_some();
            if !in_query && !in_payload {
                return Err(ProcessUriError::MissingDomain(format!(
                    "Missing required parameter '{}' for action '{}'",
                    req, uri.canonical_action()
                )));
            }
        }
        Ok(())
    }
}
