use app_config::AppConfigError;
use daemon_client::{ErrorResponse, ProjectInfo, ServiceInfo, ServiceStatus};
use project_settings::ProjectSettingsError;
use std::{error::Error, vec};

pub enum Output {
    Stdout(String),
    Stderr(String),
}

impl Output {
    pub fn socket_not_alive(socket_path: &str) -> Self {
        Self::Stderr(format!(
            "Cannot connect to the Conc daemon at unix://{}. Daemon is not running or is using different work directory.",
            socket_path
        ))
    }
}

impl From<Result<(), Box<dyn Error>>> for Output {
    fn from(value: Result<(), Box<dyn Error>>) -> Self {
        match value {
            Ok(_) => Self::Stdout(String::new()),
            Err(err) => Self::Stderr(err.to_string()),
        }
    }
}

impl From<ErrorResponse> for Output {
    fn from(value: ErrorResponse) -> Self {
        Self::Stderr(value.to_string())
    }
}

impl From<Result<String, ErrorResponse>> for Output {
    fn from(value: Result<String, ErrorResponse>) -> Self {
        match value {
            Ok(val) => Self::Stdout(val),
            Err(err) => err.into(),
        }
    }
}

impl From<Result<(), ErrorResponse>> for Output {
    fn from(value: Result<(), ErrorResponse>) -> Self {
        match value {
            Ok(_) => Self::Stdout(String::from("success")),
            Err(err) => err.into(),
        }
    }
}

impl From<Result<Vec<String>, ErrorResponse>> for Output {
    fn from(value: Result<Vec<String>, ErrorResponse>) -> Self {
        match value {
            Ok(val) => Self::Stdout(val.join(" ")),
            Err(err) => err.into(),
        }
    }
}

impl From<Result<ServiceInfo, ErrorResponse>> for Output {
    fn from(value: Result<ServiceInfo, ErrorResponse>) -> Self {
        match value {
            Ok(val) => Self::Stdout(format_services_info(vec![val])),
            Err(err) => err.into(),
        }
    }
}

impl From<Result<ProjectInfo, ErrorResponse>> for Output {
    fn from(value: Result<ProjectInfo, ErrorResponse>) -> Self {
        match value {
            Ok(val) => Self::Stdout(format_project_info(val)),
            Err(err) => err.into(),
        }
    }
}

impl From<Result<Vec<ProjectInfo>, ErrorResponse>> for Output {
    fn from(value: Result<Vec<ProjectInfo>, ErrorResponse>) -> Self {
        match value {
            Ok(val) => Self::Stdout(format_projects_info(val)),
            Err(err) => err.into(),
        }
    }
}

impl From<ProjectSettingsError> for Output {
    fn from(value: ProjectSettingsError) -> Self {
        Self::Stderr(value.to_string())
    }
}

impl From<AppConfigError> for Output {
    fn from(value: AppConfigError) -> Self {
        Self::Stderr(value.to_string())
    }
}

fn format_projects_info(projects: Vec<ProjectInfo>) -> String {
    let mut output = vec![];

    for project in projects {
        output.push(format_project_info(project));
    }

    if output.is_empty() {
        return String::from("No project was found.");
    }

    output.join("\n\n")
}

fn format_project_info(project: ProjectInfo) -> String {
    let mut output = format!(
        "Project: {}, {}/{} Running\n",
        project.name,
        project.running_service_count(),
        project.service_count()
    );
    let service_table = format_services_info(project.services);
    output.push_str(&service_table);

    output
}

fn format_services_info(services: Vec<ServiceInfo>) -> String {
    let mut service_names_column = vec![String::from("NAME")];
    let mut service_statuses_column = vec![String::from("STATUS")];
    let mut service_pids_column = vec![String::from("PID")];

    for service in services {
        service_names_column.push(service.name);
        service_statuses_column.push(format_service_status(service.status));
        service_pids_column.push(service.pid.to_string());
    }

    format_table(vec![
        service_names_column,
        service_statuses_column,
        service_pids_column,
    ])
}

fn format_service_status(service_status: ServiceStatus) -> String {
    match service_status {
        ServiceStatus::IDLE => String::from("IDLE"),
        ServiceStatus::RUNNING => String::from("RUNNING"),
        ServiceStatus::STOPPED => String::from("STOPPED"),
    }
}

fn format_table(columns: Vec<Vec<String>>) -> String {
    if columns.is_empty() {
        return String::new();
    };

    let mut rows: Vec<Vec<String>> = vec![vec![]; columns[0].len()];

    for column in columns {
        let max_len = column
            .iter()
            .map(|item| item.len())
            .max()
            .unwrap_or_default();

        for (i, element) in column.iter().enumerate() {
            let mut new_element = String::from(element);

            let padding_cnt = max_len - element.len();
            if padding_cnt > 0 {
                let spaces = vec![' '; padding_cnt];
                new_element.push_str(&spaces.into_iter().collect::<String>());
            }

            rows[i].push(new_element);
        }
    }

    rows.iter()
        .map(|row| row.join(" "))
        .collect::<Vec<String>>()
        .join("\n")
}
