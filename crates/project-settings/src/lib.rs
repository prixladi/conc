use serde::{Deserialize, Serialize, Serializer};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryFrom,
    path::Path,
};

const SETTINGS_FILE_RELATIVE_NAME: &str = "conc.json";

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
    #[error("project name is invalid, characters must be alphanumeric or '-' or '_'")]
    InvalidProjectName,
    #[error("no service was specified")]
    EmptyServices,
    #[error("service name is empty")]
    EmptyServiceName,
    #[error("service '{service_name}' is invalid, characters must be alphanumeric or '-' or '_'")]
    InvalidServiceName { service_name: String },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvValue {
    Str(String),
    Num(i32),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectSettings {
    pub name: String,
    #[serde(default = "String::new")]
    pub cwd: String,
    pub services: Vec<ServiceSettings>,
    #[serde(
        default = "HashMap::new",
        serialize_with = "ordered_map",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub env: HashMap<String, EnvValue>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceSettings {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pwd: Option<String>,
    pub command: Vec<String>,
    #[serde(
        default = "HashMap::new",
        serialize_with = "ordered_map",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub env: HashMap<String, EnvValue>,
}

impl TryFrom<&ProjectSettings> for String {
    type Error = ProjectSettingsError;

    fn try_from(value: &ProjectSettings) -> Result<Self, Self::Error> {
        let json = serde_json::to_string(value)?;
        Ok(json)
    }
}

impl ProjectSettings {
    pub fn prettify_json(data: &str) -> Result<String, serde_json::Error> {
        serde_json::from_str::<Self>(data).and_then(|d| serde_json::to_string_pretty(&d))
    }
}

impl ProjectSettings {
    pub fn find_parse_and_populate(pwd: Option<String>) -> Result<Self, ProjectSettingsError> {
        let (path, json) = resolve_settings_path_and_json(pwd)?;
        let mut settings = try_parse(json.as_str())?;

        settings.cwd = resolve_cwd(path, settings.cwd);
        settings.env = populate_env(settings.env);

        for service in &mut settings.services {
            service.env = populate_env(service.env.clone());
        }

        Ok(settings)
    }
}

fn resolve_cwd(settings_path: String, provided_cwd: String) -> String {
    if provided_cwd.is_empty() {
        return settings_path;
    }

    let cwd_path = Path::new(&provided_cwd);
    if cwd_path.is_absolute() {
        return provided_cwd;
    };

    String::from(Path::new(&settings_path).join(cwd_path).to_str().unwrap())
}

fn try_parse(value: &str) -> Result<ProjectSettings, ProjectSettingsError> {
    let settings = serde_json::from_str::<ProjectSettings>(value)?;

    if settings.name.is_empty() {
        return Err(ProjectSettingsError::EmptyProjectName);
    }

    if !is_name_valid(&settings.name) {
        return Err(ProjectSettingsError::InvalidProjectName);
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

        if !is_name_valid(&name) {
            return Err(ProjectSettingsError::InvalidServiceName { service_name: name });
        }

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

fn resolve_settings_path_and_json(
    pwd: Option<String>,
) -> Result<(String, String), ProjectSettingsError> {
    let mut path = match pwd {
        Some(pwd) => {
            let path = Path::new(&pwd);
            match path.is_absolute() {
                true => Ok(path.to_path_buf()),
                false => std::env::current_dir().map(|cd| cd.join(path)),
            }
        }
        None => std::env::current_dir(),
    }?;

    if path.is_dir() {
        path = path.join(SETTINGS_FILE_RELATIVE_NAME);
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

fn populate_env(envs: HashMap<String, EnvValue>) -> HashMap<String, EnvValue> {
    envs.into_iter()
        .map(|(key, value)| {
            let val = match value {
                EnvValue::Str(value) => value,
                EnvValue::Num(value) => value.to_string(),
            };

            (key, EnvValue::Str(val))
        })
        .collect()
}

fn is_name_valid(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

fn ordered_map<S: Serializer, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
