use std::{convert::From, convert::TryFrom, vec};

#[derive(Debug, thiserror::Error)]
pub enum ErrorResponse {
    #[error("Error occurred while trying to communicate with daemon socket: {inner}")]
    Socket { inner: std::io::Error },
    #[error("Unexpected error ocurred on the client side: {0}")]
    Client(String),
    #[error("Unexpected error ocurred in the daemon, check its logs for more info: {0}")]
    Daemon(String),
    #[error("Unable to parse daemon response: {0}")]
    Malformed(String),
    #[error("Provided project was not found.")]
    ProjectNotFound(String),
    #[error("Provided service was not found in provided project.")]
    ServiceNotFound(String),
}

impl From<std::io::Error> for ErrorResponse {
    fn from(error: std::io::Error) -> Self {
        Self::Socket { inner: error }
    }
}

impl From<Vec<String>> for ErrorResponse {
    fn from(data: Vec<String>) -> Self {
        let raw = data.join("\n");

        if data.is_empty() || data[0] != "ERROR" {
            return Self::Malformed(raw);
        }

        let err_name = if data.len() > 1 { &data[1] } else { "" };
        match err_name {
            "project_not_found" => Self::ProjectNotFound(raw),
            "service_not_found" => Self::ServiceNotFound(raw),
            x if x.starts_with("settings.")
                || x == "unknown_command"
                || x == "invalid_argument_count" =>
            {
                Self::Client(raw)
            }
            x if x.starts_with("unknown-code-") || x == "driver_error" || x == "manager_error" => {
                Self::Daemon(raw)
            }
            _ => Self::Malformed(raw),
        }
    }
}

#[derive(Debug)]
pub struct NameListResponse {
    pub values: Vec<String>,
}

pub trait Response: TryFrom<Vec<String>> {}

impl TryFrom<Vec<String>> for NameListResponse {
    type Error = ();

    fn try_from(data: Vec<String>) -> Result<Self, Self::Error> {
        if data.is_empty() || data[0] != "OK" {
            return Err(());
        }

        let names = &data[1..];
        Ok(Self {
            values: Vec::from(names),
        })
    }
}
impl Response for NameListResponse {}

#[derive(Debug)]
pub struct ProjectSettingsResponse {
    pub value: String,
}

impl TryFrom<Vec<String>> for ProjectSettingsResponse {
    type Error = ();

    fn try_from(data: Vec<String>) -> Result<Self, Self::Error> {
        if data.len() != 2 || data[0] != "OK" {
            return Err(());
        }

        Ok(Self {
            value: String::from(&data[1]),
        })
    }
}
impl Response for ProjectSettingsResponse {}

#[derive(Debug)]
pub struct ProjectsSettingsResponse {
    pub values: Vec<(String, String)>,
}

impl TryFrom<Vec<String>> for ProjectsSettingsResponse {
    type Error = ();

    fn try_from(data: Vec<String>) -> Result<Self, Self::Error> {
        if data.is_empty() || data[0] != "OK" {
            return Err(());
        }

        let lines = &data[1..];
        let mut values = vec![];

        for line in lines.iter() {
            let parts: Vec<&str> = line.split("").collect();
            if parts.len() != 2 {
                return Err(());
            }
            values.push((String::from(parts[0]), String::from(parts[1])));
        }

        Ok(Self { values })
    }
}
impl Response for ProjectsSettingsResponse {}

#[derive(Debug)]
pub struct ServiceInfoResponse {
    pub value: ServiceInfo,
}

impl TryFrom<Vec<String>> for ServiceInfoResponse {
    type Error = ();

    fn try_from(data: Vec<String>) -> Result<Self, Self::Error> {
        if data.len() != 2 || data[0] != "OK" {
            return Err(());
        }

        Ok(Self {
            value: ServiceInfo::try_from(data[1].as_str())?,
        })
    }
}
impl Response for ServiceInfoResponse {}

#[derive(Debug)]
pub struct ProjectInfoResponse {
    pub value: ProjectInfo,
}

impl TryFrom<Vec<String>> for ProjectInfoResponse {
    type Error = ();

    fn try_from(data: Vec<String>) -> Result<Self, Self::Error> {
        if data.len() < 2 || data[0] != "OK" {
            return Err(());
        }

        if data[1].contains(" ") {
            return Err(());
        }

        let mut value = ProjectInfo {
            name: data[1].clone(),
            services: vec![],
        };

        for line in &data[2..] {
            let service_info = ServiceInfo::try_from(line.as_str())?;
            value.services.push(service_info);
        }

        Ok(Self { value })
    }
}
impl Response for ProjectInfoResponse {}

#[derive(Debug)]
pub struct ProjectsInfoResponse {
    pub values: Vec<ProjectInfo>,
}

impl TryFrom<Vec<String>> for ProjectsInfoResponse {
    type Error = ();

    fn try_from(data: Vec<String>) -> Result<Self, Self::Error> {
        if data.is_empty() || data[0] != "OK" {
            return Err(());
        }

        let mut values = vec![];
        for line in &data[1..] {
            if !line.contains(" ") {
                let project_info = ProjectInfo {
                    name: String::from(line),
                    services: vec![],
                };
                values.push(project_info);
                continue;
            }

            // first element should always be a project name so if we reach this message is malformed
            if values.is_empty() {
                return Err(());
            }

            let service_info = ServiceInfo::try_from(line.as_str())?;
            let current_index = values.len() - 1;
            values[current_index].services.push(service_info);
        }

        Ok(Self { values })
    }
}
impl Response for ProjectsInfoResponse {}

#[derive(Debug)]
pub struct NoContentResponse;

impl TryFrom<Vec<String>> for NoContentResponse {
    type Error = ();

    fn try_from(data: Vec<String>) -> Result<Self, Self::Error> {
        if data.is_empty() || data[0] != "OK" {
            return Err(());
        }

        Ok(Self)
    }
}
impl Response for NoContentResponse {}

#[derive(Debug, PartialEq, Clone)]
pub enum ServiceStatus {
    IDLE,
    RUNNING,
    STOPPED,
}

impl TryFrom<&str> for ServiceStatus {
    type Error = ();

    fn try_from(data: &str) -> Result<Self, Self::Error> {
        match data {
            "IDLE" => Ok(Self::IDLE),
            "RUNNING" => Ok(Self::RUNNING),
            "STOPPED" => Ok(Self::STOPPED),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub services: Vec<ServiceInfo>,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub logfile_path: String,
    pub pid: i32,
}

impl TryFrom<&str> for ServiceInfo {
    type Error = ();

    fn try_from(data: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.split(" ").collect();
        if parts.len() < 4 {
            return Err(());
        }

        let status = ServiceStatus::try_from(parts[1])?;
        let pid = parts[2].parse::<i32>().map_err(|_| ())?;

        let logfile_path = if parts[3] == "-" {
            "/dev/null"
        } else {
            parts[3]
        };

        Ok(Self {
            name: String::from(parts[0]),
            status,
            logfile_path: String::from(logfile_path),
            pid,
        })
    }
}
