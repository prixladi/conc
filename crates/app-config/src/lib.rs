use serde::{Deserialize, Serialize};
use std::path::Path;

const HOME_ENV_VAR: &str = "HOME";
const CONF_RELATIVE_LOCATION: &str = ".conc/conf.json";
const SOCKET_DEBUG_LOCATION: &str = "../daemon/run/conc.sock";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub daemon_socket_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppConfigError {
    #[error("Unable to read 'HOME' env variable was not set. Error: {inner}")]
    HomeNotFound { inner: std::env::VarError },
    #[error("Unable to read configuration file '{path}'. Error: {inner}")]
    ConfigFileNotFound { path: String, inner: std::io::Error },
    #[error("Unable to parse config from configuration file '{path}'. Error: {inner}")]
    ConfigFileNotParsable {
        path: String,
        inner: serde_json::Error,
    },
}

impl AppConfig {
    pub fn new() -> Result<Self, AppConfigError> {
        if cfg!(debug_assertions) {
            return Ok(Self {
                daemon_socket_path: String::from(SOCKET_DEBUG_LOCATION),
            });
        }

        let home_dir =
            std::env::var(HOME_ENV_VAR).map_err(|e| AppConfigError::HomeNotFound { inner: e })?;
        let conc_config = Path::new(&home_dir)
            .join(CONF_RELATIVE_LOCATION)
            .to_str()
            .unwrap()
            .to_string();

        let data = std::fs::read_to_string(&conc_config).map_err(|e| {
            AppConfigError::ConfigFileNotFound {
                path: conc_config.clone(),
                inner: e,
            }
        })?;

        serde_json::from_str(&data).map_err(|e| AppConfigError::ConfigFileNotParsable {
            path: conc_config.clone(),
            inner: e,
        })
    }
}