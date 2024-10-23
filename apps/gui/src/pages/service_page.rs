use std::path::Path;

use daemon_client::{Requester, ServiceInfo};
use iced::widget::{column, scrollable, text};
use iced::{Element, Length};

use crate::components::Section;
use crate::message::Message;

use super::{Page, PageData, PageView};

pub struct ServicePage {
    requester: Requester,
    project_name: String,
    service_name: String,
    service: Option<ServiceInfo>,
}

impl ServicePage {
    pub fn new(data: PageData, project_name: String, service_name: String) -> Self {
        Self {
            requester: data.requester,
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

    fn refresh(&mut self, _: PageData) -> Result<(), String> {
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
            let title = Section::new(text(self.title()).size(30).into());
            view = view.push(title);

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
            let log_section = Section::new(log_data_view.into());
            view = view.push(log_section);
        }

        view.into()
    }
}
