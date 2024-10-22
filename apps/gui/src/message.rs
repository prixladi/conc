use iced::Theme;

use crate::pages::Page;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    GotoPage(Page),
    OpenUrl(String),
    ThemeChanged(Theme),
    CopyToClipboard {
        name: String,
        data: String,
    },
    RefreshLoop,
    StartProject {
        project_name: String,
    },
    RestartProject {
        project_name: String,
    },
    StopProject {
        project_name: String,
    },
    StartService {
        project_name: String,
        service_name: String,
    },
    RestartService {
        project_name: String,
        service_name: String,
    },
    StopService {
        project_name: String,
        service_name: String,
    },
}

impl Message {
    pub fn to_success_message(&self) -> String {
        match self {
            Message::GotoPage(page) => format!("Navigated to the page '{}'", page),
            Message::OpenUrl(url) => format!("Opened the external url '{}'", url),
            Message::ThemeChanged(theme) => format!("Changed theme to '{}'", theme),
            Message::RefreshLoop => String::from("Performed the refresh loop"),
            Message::StartProject { project_name } => {
                format!("Started the project '{}'", project_name)
            }
            Message::RestartProject { project_name } => {
                format!("Restarted the project '{}'", project_name)
            }
            Message::StopProject { project_name } => {
                format!("Stopped the project '{}'", project_name)
            }
            Message::StartService {
                project_name,
                service_name,
            } => format!("Started the service '{}/{}'", project_name, service_name),
            Message::RestartService {
                project_name,
                service_name,
            } => format!("Restarted the service '{}/{}'", project_name, service_name),
            Message::StopService {
                project_name,
                service_name,
            } => format!("Restarted the service '{}/{}'", project_name, service_name),
            Message::CopyToClipboard { name, data: _ } => {
                format!("Copied '{}' to the clipboard.", name)
            }
        }
    }

    pub fn to_error_message(&self, error: &str) -> String {
        let message = match self {
            Message::GotoPage(page) => format!("Unable to navigate to the page '{}'", page),
            Message::OpenUrl(url) => format!("Opened the external url '{}'", url),
            Message::ThemeChanged(theme) => format!("Changed theme to '{}'", theme),
            Message::RefreshLoop => String::from("Unable to perform the refresh loop"),
            Message::StartProject { project_name } => {
                format!("Unable to start the project '{}'", project_name)
            }
            Message::RestartProject { project_name } => {
                format!("Unable to restart the project '{}'", project_name)
            }
            Message::StopProject { project_name } => {
                format!("Unable to stop the project '{}'", project_name)
            }
            Message::StartService {
                project_name,
                service_name,
            } => format!(
                "Unable to start the service '{}/{}'",
                project_name, service_name
            ),
            Message::RestartService {
                project_name,
                service_name,
            } => format!(
                "Unable to restart the service '{}/{}'",
                project_name, service_name
            ),
            Message::StopService {
                project_name,
                service_name,
            } => format!(
                "Unable to stop the service '{}/{}'",
                project_name, service_name
            ),
            Message::CopyToClipboard { name, data: _ } => {
                format!("Unable to copy '{}' to the clipboard.", name)
            }
        };

        format!("{}, error: {}", message, error)
    }
}
