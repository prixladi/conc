use serde::{Deserialize, Serialize};
use std::path::Path;

const HOME_ENV_VAR: &str = "HOME";
const CONF_RELATIVE_LOCATION: &str = ".conc/conf.json";
const SOCKET_RELATIVE_LOCATION: &str = ".conc/run/conc.sock";
const SOCKET_DEBUG_LOCATION: &str = "../daemon/run/conc.sock";

#[derive(Debug, Clone, Serialize)]
pub struct AppConfig {
    pub use_caller_env: bool,
    pub daemon_socket_path: String,
    pub log_view_command: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct UserAppConfig {
    pub use_caller_env: Option<bool>,
    pub daemon_socket_path: Option<String>,
    pub log_view_command: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppConfigError {
    #[error("Unable to read 'HOME' env variable was not set. Error: {inner}")]
    HomeNotFound { inner: std::env::VarError },
    #[error("Unable to parse config from configuration file '{path}'. Error: {inner}")]
    ConfigFileNotParsable {
        path: String,
        inner: serde_json::Error,
    },
}

impl From<std::env::VarError> for AppConfigError {
    fn from(value: std::env::VarError) -> Self {
        AppConfigError::HomeNotFound { inner: value }
    }
}

impl AppConfig {
    pub fn new() -> Result<Self, AppConfigError> {
        if cfg!(debug_assertions) {
            return Ok(Self {
                use_caller_env: true,
                daemon_socket_path: String::from(SOCKET_DEBUG_LOCATION),
                log_view_command: get_default_log_view_command(),
            });
        }

        let home_dir = std::env::var(HOME_ENV_VAR)?;
        let conc_config = get_path_in_home(&home_dir, CONF_RELATIVE_LOCATION);

        std::fs::read_to_string(&conc_config)
            .map(|data| {
                serde_json::from_str(&data).map_err(|e| AppConfigError::ConfigFileNotParsable {
                    path: conc_config,
                    inner: e,
                })
            })
            .unwrap_or(Ok(UserAppConfig::default()))
            .map(|uc| AppConfig {
                use_caller_env: uc.use_caller_env.unwrap_or(true),
                daemon_socket_path: uc
                    .daemon_socket_path
                    .unwrap_or(get_path_in_home(&home_dir, SOCKET_RELATIVE_LOCATION)),
                log_view_command: uc
                    .log_view_command
                    .unwrap_or_else(get_default_log_view_command),
            })
    }
}

fn get_path_in_home(home_dir: &str, path: &str) -> String {
    Path::new(home_dir).join(path).to_str().unwrap().to_string()
}

fn get_default_log_view_command() -> Vec<String> {
    vec![
        String::from("less"),
        String::from("-R"),
        String::from("+GF"),
    ]
}
