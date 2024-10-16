use serde::{Deserialize, Serialize};
use std::path::Path;

const HOME_ENV_VAR: &str = "HOME";
const CLI_CONF_RELATIVE_LOCATION: &str = ".conc/cli-conf.json";
const CLI_CONF_DEBUG_LOCATION: &str = "../daemon/run/conc.sock";

#[derive(Debug, Deserialize, Serialize)]
pub struct CliConfig {
    pub daemon_socket_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CliConfigError {
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

impl CliConfig {
    pub fn new() -> Result<Self, CliConfigError> {
        if cfg!(debug_assertions) {
            return Ok(Self {
                daemon_socket_path: String::from(CLI_CONF_DEBUG_LOCATION),
            });
        }

        let home_dir =
            std::env::var(HOME_ENV_VAR).map_err(|e| CliConfigError::HomeNotFound { inner: e })?;
        let conc_config = Path::new(&home_dir)
            .join(CLI_CONF_RELATIVE_LOCATION)
            .to_str()
            .unwrap()
            .to_string();

        let data = std::fs::read_to_string(&conc_config).map_err(|e| {
            CliConfigError::ConfigFileNotFound {
                path: conc_config.clone(),
                inner: e,
            }
        })?;

        serde_json::from_str(&data).map_err(|e| CliConfigError::ConfigFileNotParsable {
            path: conc_config.clone(),
            inner: e,
        })
    }
}
