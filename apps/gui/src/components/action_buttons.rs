use daemon_client::{ProjectInfo, ServiceInfo, ServiceStatus};
use iced::{
    widget::{button, row},
    Element, Padding,
};

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
        let restart_button = button("Restart")
            .style(button::primary)
            .padding(Padding::default().top(7).bottom(2).left(4).right(4))
            .on_press(Message::RestartService {
                project: self.project_name.to_string(),
                name: self.service.name.clone(),
            });

        let mut start_button = button("Start")
            .style(button::success)
            .padding(Padding::default().top(7).bottom(2).left(4).right(4));
        if self.service.status != ServiceStatus::RUNNING {
            start_button = start_button.on_press(Message::StartService {
                project: self.project_name.to_string(),
                name: self.service.name.clone(),
            });
        };

        let mut stop_button = button("Stop")
            .style(button::danger)
            .padding(Padding::default().top(7).bottom(2).left(4).right(4));
        if self.service.status == ServiceStatus::RUNNING {
            stop_button = stop_button.on_press(Message::StopService {
                project: self.project_name.to_string(),
                name: self.service.name.clone(),
            });
        };

        row![start_button, stop_button, restart_button]
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
        let running_services_count = self.project
            .services
            .iter()
            .filter(|service| service.status == ServiceStatus::RUNNING)
            .count();

        let restart_button = button("Restart")
            .style(button::primary)
            .padding(Padding::default().top(7).bottom(2).left(4).right(4))
            .on_press(Message::RestartProject {
                name: self.project.name.clone(),
            });

        let mut start_button = button("Start")
            .style(button::success)
            .padding(Padding::default().top(7).bottom(2).left(4).right(4));
        if services_count > running_services_count {
            start_button = start_button.on_press(Message::StartProject {
                name: self.project.name.clone(),
            });
        };

        let mut stop_button = button("Stop")
            .style(button::danger)
            .padding(Padding::default().top(7).bottom(2).left(4).right(4));
        if running_services_count > 0 {
            stop_button = stop_button.on_press(Message::StopProject {
                name: self.project.name.clone(),
            });
        };

        row![start_button, stop_button, restart_button]
            .spacing(10)
            .into()
    }
}
