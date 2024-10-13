use serde::{Deserialize, Serialize};
use std::{collections::{HashMap, HashSet}, convert::TryFrom};

#[derive(Debug, thiserror::Error)]
pub enum SettingsValidationError {
    #[error("unable to parse the settings: {inner}")]
    ParserError { inner: serde_json::Error },
    #[error("project name is empty")]
    EmptyProjectName,
    #[error("no service was specified")]
    EmptyServices,
    #[error("service name is empty")]
    EmptyServiceName,
    #[error("service '{service_name}' is declared more than once")]
    DuplicateServiceName { service_name: String },
    #[error("service '{service_name}' has an empty command")]
    EmptyCommand { service_name: String },
}

impl From<serde_json::Error> for SettingsValidationError {
    fn from(value: serde_json::Error) -> Self {
        SettingsValidationError::ParserError { inner: value }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectSettings {
    pub name: String,
    pub services: Vec<ServiceSettings>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceSettings {
    pub name: String,
    pub pwd: Option<String>,
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
}

impl TryFrom<&str> for ProjectSettings {
    type Error = SettingsValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let settings = serde_json::from_str::<ProjectSettings>(value)?;

        if settings.name.is_empty() {
            return Err(SettingsValidationError::EmptyProjectName);
        }
        if settings.services.is_empty() {
            return Err(SettingsValidationError::EmptyServices);
        }
    
        let mut name_cache = HashSet::with_capacity(settings.services.len());
    
        for service in &settings.services {
            if service.name.is_empty() {
                return Err(SettingsValidationError::EmptyServiceName);
            }
    
            let name = service.name.clone();
            if service.command.is_empty() {
                return Err(SettingsValidationError::EmptyCommand { service_name: name });
            }
            if name_cache.contains(&name) {
                return Err(SettingsValidationError::DuplicateServiceName { service_name: name });
            }
    
            name_cache.insert(name);
        }
    
        Ok(settings)
    }
}
