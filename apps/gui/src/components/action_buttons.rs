use daemon_client::{ProjectInfo, ServiceInfo, ServiceStatus};
use iced::{
    widget::{button, button::Status, row, text},
    Background, Border, Color, Element, Shadow, Theme,
};
use iced_fonts::{bootstrap::icon_to_string, Bootstrap, BOOTSTRAP_FONT};

use crate::message::Message;

pub struct ServiceActions<'a> {
    project_name: &'a str,
    service: &'a ServiceInfo,
}

impl<'a> ServiceActions<'a> {
    pub fn new(project_name: &'a str, service: &'a ServiceInfo) -> Self {
        Self {
            project_name,
            service,
        }
    }

    pub fn render(self) -> Element<'a, Message> {
        let mut start_message = None;
        if self.service.status != ServiceStatus::RUNNING {
            start_message = Some(Message::StartService {
                project_name: self.project_name.to_string(),
                service_name: self.service.name.clone(),
            });
        };

        let mut stop_message = None;
        if self.service.status == ServiceStatus::RUNNING {
            stop_message = Some(Message::StopService {
                project_name: self.project_name.to_string(),
                service_name: self.service.name.clone(),
            });
        };

        let restart_message = Some(Message::RestartService {
            project_name: self.project_name.to_string(),
            service_name: self.service.name.clone(),
        });

        row![
            start_action_button(start_message),
            stop_action_button(stop_message),
            restart_action_button(restart_message),
        ]
        .spacing(10)
        .into()
    }
}

pub struct ProjectActions<'a> {
    project: &'a ProjectInfo,
}

impl<'a> ProjectActions<'a> {
    pub fn new(project: &'a ProjectInfo) -> Self {
        Self { project }
    }

    pub fn render(self) -> Element<'a, Message> {
        let services_count = self.project.services.len();
        let running_services_count = self
            .project
            .services
            .iter()
            .filter(|service| service.status == ServiceStatus::RUNNING)
            .count();

        let mut start_message = None;
        if services_count > running_services_count {
            start_message = Some(Message::StartProject {
                project_name: self.project.name.clone(),
            });
        };

        let mut stop_message = None;
        if running_services_count > 0 {
            stop_message = Some(Message::StopProject {
                project_name: self.project.name.clone(),
            });
        }

        let restart_message = Some(Message::RestartProject {
            project_name: self.project.name.clone(),
        });

        row![
            start_action_button(start_message),
            stop_action_button(stop_message),
            restart_action_button(restart_message),
        ]
        .spacing(10)
        .into()
    }
}

fn start_action_button<'a>(message: Option<Message>) -> Element<'a, Message> {
    action_button(message, Bootstrap::PlayFill, 25, start_action_button_style)
}

fn stop_action_button<'a>(message: Option<Message>) -> Element<'a, Message> {
    action_button(message, Bootstrap::StopFill, 25, stop_action_button_style)
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
        palette.success.weak.color,
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
