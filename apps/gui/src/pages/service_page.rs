use std::fs::File;
use std::io::Read;
use std::path::Path;

use daemon_client::{ProjectInfo, Requester, ServiceInfo};
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length};
use project_settings::ProjectSettings;

use crate::components::{InfoTable, Section, ServiceActions};
use crate::message::Message;
use crate::utils::service_status_stringify;

use super::{Page, PageView};

pub struct ServicePage {
    requester: Requester,
    project_name: String,
    service_name: String,
    service: Option<ServiceInfo>,
}

impl ServicePage {
    pub fn new(requester: Requester, project_name: String, service_name: String) -> Self {
        Self {
            requester,
            project_name,
            service_name,
            service: None,
        }
    }
}

impl PageView for ServicePage {
    fn page(&self) -> Page {
        Page::Service(self.project_name.clone(), self.project_name.clone())
    }

    fn title(&self) -> String {
        format!("Service - {}/{}", self.project_name, self.service_name)
    }

    fn refresh(&mut self) -> Result<(), String> {
        let result = self
            .requester
            .get_services_info(&self.project_name, &self.service_name);

        match result {
            Ok(service) => {
                self.service = Some(service.value);
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn view(&self) -> Element<Message> {
        let mut view = column![];

        if let Some(service) = &self.service {
            view = view.push(Section::new().render(text(self.title()).size(30)));

            // TODO: Use some more efficient way to read just last n lines
            let log_data = std::fs::read_to_string(Path::new(&service.logfile_path))
                .map(|text| {
                    text.lines()
                        .rev()
                        .take(200)
                        .collect::<Vec<&str>>()
                        .join("\n")
                })
                .unwrap_or_default();

            let log_data_view = scrollable(text(log_data).width(Length::Fill));
            let log_section = Section::new().render(log_data_view);
            view = view.push(log_section);
        }

        view.into()
    }
}
