use daemon_client::{ProjectInfo, ServiceInfo, ServiceStatus};
use iced::{
    widget::{button, button::Status, row, text},
    Background, Border, Color, Element, Shadow, Theme,
};
use iced_fonts::{bootstrap::icon_to_string, Bootstrap, BOOTSTRAP_FONT};

use crate::message::Message;

pub struct ServiceActionButtons<'a> {
    project_name: &'a str,
    service: &'a ServiceInfo,
}

impl<'a> ServiceActionButtons<'a> {
    pub fn new(project_name: &'a str, service: &'a ServiceInfo) -> Self {
        Self {
            project_name,
            service,
        }
    }
}

impl<'a> From<ServiceActionButtons<'a>> for Element<'a, Message> {
    fn from(value: ServiceActionButtons<'a>) -> Self {
        let start_message = match value.service.status {
            ServiceStatus::RUNNING => None,
            _ => Some(Message::StartService {
                project_name: value.project_name.to_string(),
                service_name: value.service.name.clone(),
            }),
        };

        let stop_message = match value.service.status {
            ServiceStatus::RUNNING => Some(Message::StopService {
                project_name: value.project_name.to_string(),
                service_name: value.service.name.clone(),
            }),
            _ => None,
        };

        let restart_message = match value.service.status {
            ServiceStatus::RUNNING => Some(Message::RestartService {
                project_name: value.project_name.to_string(),
                service_name: value.service.name.clone(),
            }),
            _ => None,
        };

        row![
            start_action_button(start_message),
            stop_action_button(stop_message),
            restart_action_button(restart_message),
        ]
        .spacing(10)
        .into()
    }
}

pub struct ProjectActionButtons<'a> {
    project: &'a ProjectInfo,
}

impl<'a> ProjectActionButtons<'a> {
    pub fn new(project: &'a ProjectInfo) -> Self {
        Self { project }
    }
}

impl<'a> From<ProjectActionButtons<'a>> for Element<'a, Message> {
    fn from(value: ProjectActionButtons<'a>) -> Self {
        let services_count = value.project.services.len();
        let running_services_count = value
            .project
            .services
            .iter()
            .filter(|service| service.status == ServiceStatus::RUNNING)
            .count();

        let start_message = match services_count > running_services_count {
            true => Some(Message::StartProject {
                project_name: value.project.name.clone(),
            }),
            false => None,
        };

        let stop_message = match running_services_count > 0 {
            true => Some(Message::StopProject {
                project_name: value.project.name.clone(),
            }),
            false => None,
        };

        let restart_message = match running_services_count > 0 {
            true => Some(Message::RestartProject {
                project_name: value.project.name.clone(),
            }),
            false => None,
        };

        row![
            start_action_button(start_message),
            stop_action_button(stop_message),
            restart_action_button(restart_message),
        ]
        .spacing(10)
        .into()
    }
}

pub struct CopyToClipboardButton {
    name: String,
    data: String,
}

impl CopyToClipboardButton {
    pub fn new(name: String, data: String) -> Self {
        Self { name, data }
    }
}

impl<'a> From<CopyToClipboardButton> for Element<'a, Message> {
    fn from(value: CopyToClipboardButton) -> Self {
        action_button(
            Some(Message::CopyToClipboard {
                name: value.name,
                data: value.data,
            }),
            Bootstrap::ClipboardCheck,
            25,
            |theme, status| {
                let palette = theme.extended_palette();
                action_button_style(
                    status,
                    palette.secondary.base.text.scale_alpha(0.9),
                    palette.secondary.base.text,
                )
            },
        )
    }
}

fn start_action_button<'a>(message: Option<Message>) -> Element<'a, Message> {
    action_button(
        message,
        Bootstrap::PlayCircle,
        25,
        start_action_button_style,
    )
}

fn stop_action_button<'a>(message: Option<Message>) -> Element<'a, Message> {
    action_button(message, Bootstrap::StopCircle, 25, stop_action_button_style)
}

fn restart_action_button<'a>(message: Option<Message>) -> Element<'a, Message> {
    action_button(
        message,
        Bootstrap::ArrowClockwise,
        25,
        restart_action_button_style,
    )
}

fn action_button<'a>(
    message: Option<Message>,
    icon: Bootstrap,
    icon_size: i32,
    style: impl Fn(&Theme, Status) -> button::Style + 'a,
) -> Element<'a, Message> {
    let mut action = button(
        text(icon_to_string(icon))
            .size(icon_size as f32)
            .font(BOOTSTRAP_FONT),
    )
    .style(style)
    .padding(0);

    if let Some(message) = message {
        action = action.on_press(message);
    };

    action.into()
}

fn start_action_button_style(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.extended_palette();
    action_button_style(
        status,
        palette.success.strong.color.scale_alpha(0.85),
        palette.success.strong.color,
    )
}

fn stop_action_button_style(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.extended_palette();
    action_button_style(
        status,
        palette.danger.base.color,
        palette.danger.strong.color,
    )
}

fn restart_action_button_style(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.extended_palette();
    action_button_style(
        status,
        palette.background.base.text,
        palette.background.weak.text,
    )
}

fn action_button_style(
    status: Status,
    text_color: Color,
    active_text_color: Color,
) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Pressed => button::Style {
            text_color: active_text_color,
            ..base
        },
        Status::Disabled => button::Style {
            text_color: text_color.scale_alpha(0.2),
            ..base
        },
    }
}
