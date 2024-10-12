use daemon_client::{
    ErrorResponse, NoContentResponse, ProjectInfoResponse, ProjectsInfoResponse, ServiceInfo,
    ServiceInfoResponse, ServiceStatus,
};
use std::vec;

pub trait Outputable {
    fn to_output(self) -> String;
}

impl Outputable for NoContentResponse {
    fn to_output(self) -> String {
        String::from("success")
    }
}

impl Outputable for ServiceInfoResponse {
    fn to_output(self) -> String {
        format_services_info(vec![self.value])
    }
}

impl Outputable for ProjectInfoResponse {
    fn to_output(self) -> String {
        format_services_info(self.values)
    }
}

impl Outputable for ProjectsInfoResponse {
    fn to_output(self) -> String {
        format_projects_info(self.values)
    }
}

impl Outputable for ErrorResponse {
    fn to_output(self) -> String {
        format_error_response(self)
    }
}

fn format_error_response(response: ErrorResponse) -> String {
    match response {
        ErrorResponse::Client(raw) => {
            format!("{}\n{}", "Unexpected error ocurred in the cli:", raw)
        }
        ErrorResponse::Daemon(raw) => format!(
            "{}\n{}",
            "Unexpected error ocurred in the daemon, check its logs for more info:", raw
        ),
        ErrorResponse::Malformed(raw) => format!("{}\n{}", "Unable to parse daemon response:", raw),
        ErrorResponse::ProjectNotFound(_) => format!("{}", "Provided project was not found."),
        ErrorResponse::ServiceNotFound(_) => {
            format!("{}", "Provided service was not found in provided project.")
        }
    }
}

fn format_projects_info(projects: Vec<(String, Vec<ServiceInfo>)>) -> String {
    let mut output = vec![];

    for project in projects {
        output.push(format_project_info(project));
    }

    if output.len() == 0 {
        return String::from("No project was found.");
    }

    output.join("\n\n")
}

fn format_project_info(project: (String, Vec<ServiceInfo>)) -> String {
    let (project_name, services) = project;
    let services_count = services.len();
    let running_services_count = services
        .iter()
        .filter(|service| service.status == ServiceStatus::RUNNING)
        .count();

    let mut output = format!(
        "Project: {}, {}/{} Running\n",
        project_name, running_services_count, services_count
    );
    let service_table = format_services_info(services);
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
    if columns.len() == 0 {
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
