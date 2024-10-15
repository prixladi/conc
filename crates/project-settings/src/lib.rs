use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub enum ProjectSettingsError {
    #[error("error while interacting with file system:{inner}")]
    IoError { inner: std::io::Error },
    #[error("settings file does not exist: {path}")]
    NotFound { path: String },
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

impl From<serde_json::Error> for ProjectSettingsError {
    fn from(value: serde_json::Error) -> Self {
        ProjectSettingsError::ParserError { inner: value }
    }
}

impl From<std::io::Error> for ProjectSettingsError {
    fn from(value: std::io::Error) -> Self {
        ProjectSettingsError::IoError { inner: value }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectSettings {
    pub name: String,
    pub services: Vec<ServiceSettings>,
    #[serde(default = "String::new", skip_deserializing)]
    pub cwd: String,
    #[serde(default = "HashMap::new")]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceSettings {
    pub name: String,
    pub pwd: Option<String>,
    pub command: Vec<String>,
    #[serde(default = "HashMap::new")]
    pub env: HashMap<String, String>,
}

impl TryFrom<&ProjectSettings> for String {
    type Error = ProjectSettingsError;

    fn try_from(value: &ProjectSettings) -> Result<Self, Self::Error> {
        let json = serde_json::to_string(value)?;
        Ok(json)
    }
}

impl ProjectSettings {
    pub fn find_parse_and_populate(pwd: Option<String>) -> Result<Self, ProjectSettingsError> {
        let (cwd, json) = resolve_cwd_and_json(pwd)?;
        let mut setting = try_parse(json.as_str())?;
        setting.cwd = cwd;

        const PATH_VAR_NAME: &str = "PATH";
        if !setting.env.contains_key(PATH_VAR_NAME) {
            if let Ok(path) = std::env::var(PATH_VAR_NAME) {
                setting.env.insert(String::from(PATH_VAR_NAME), path);
            }
        }

        Ok(setting)
    }
}

fn try_parse(value: &str) -> Result<ProjectSettings, ProjectSettingsError> {
    let settings = serde_json::from_str::<ProjectSettings>(value)?;

    if settings.name.is_empty() {
        return Err(ProjectSettingsError::EmptyProjectName);
    }
    if settings.services.is_empty() {
        return Err(ProjectSettingsError::EmptyServices);
    }

    let mut name_cache = HashSet::with_capacity(settings.services.len());

    for service in &settings.services {
        if service.name.is_empty() {
            return Err(ProjectSettingsError::EmptyServiceName);
        }

        let name = service.name.clone();
        if service.command.is_empty() {
            return Err(ProjectSettingsError::EmptyCommand { service_name: name });
        }
        if name_cache.contains(&name) {
            return Err(ProjectSettingsError::DuplicateServiceName { service_name: name });
        }

        name_cache.insert(name);
    }

    Ok(settings)
}

fn resolve_cwd_and_json(pwd: Option<String>) -> Result<(String, String), ProjectSettingsError> {
    let mut path = match pwd {
        Some(pwd) => {
            let path = Path::new(&pwd);
            if path.is_absolute() {
                Ok(path.to_path_buf())
            } else {
                std::env::current_dir().map(|cd| cd.join(path))
            }
        }
        None => std::env::current_dir(),
    }?;

    if path.is_dir() {
        path = path.join("conc.json");
    };

    std::fs::read_to_string(&path)
        .map(|str| {
            let cwd = path
                .parent()
                .and_then(|path| path.to_str())
                .unwrap_or_default();
            (String::from(cwd), str)
        })
        .map_err(|_| ProjectSettingsError::NotFound {
            path: String::from(path.to_str().unwrap_or_default()),
        })
}
