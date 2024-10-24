use std::path::Path;

use daemon_client::ServiceInfo;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Padding};

use crate::components::{CopyToClipboardButton, PageTitle, Section, ServiceActionButtons};
use crate::message::Message;
use crate::utils::service_status_stringify;

use super::{Page, PageData, PageView};

pub struct ServicePage {
    project_name: String,
    service_name: String,
    service: Option<(ServiceInfo, String)>,
}

impl ServicePage {
    pub fn new(project_name: String, service_name: String) -> Self {
        Self {
            project_name,
            service_name,
            service: None,
        }
    }
}

impl PageView for ServicePage {
    fn page(&self) -> Page {
        Page::Service(self.project_name.clone(), self.service_name.clone())
    }

    fn refresh(&mut self, data: PageData) -> Result<(), String> {
        let result = data
            .requester
            .get_services_info(&self.project_name, &self.service_name)
            .map(|service| {
                // TODO: Use some more efficient way to read just last n lines
                let log_data = std::fs::read_to_string(Path::new(&service.value.logfile_path))
                    .map(|text| {
                        text.lines()
                            .rev()
                            .take(200)
                            .collect::<Vec<&str>>()
                            .join("\n")
                    })
                    .unwrap_or_default();

                (service.value, log_data)
            });

        match result {
            Ok(service) => {
                self.service = Some(service);
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn view(&self) -> Element<Message> {
        let mut view = column![];
        if self.service.is_none() {
            return view.into();
        };

        let (service, log_data) = self.service.as_ref().unwrap();

        let mut service_info = column![].width(Length::Fill).spacing(12);

        let title = PageTitle::new(self.title());
        service_info = service_info.push(title);

        let action_buttons = ServiceActionButtons::new(&self.project_name, service);
        let status_row = row![
            container(text("STATUS:").size(20)).padding(Padding::default().top(4)),
            container(text(service_status_stringify(&service.status)).size(18))
                .padding(Padding::default().top(4)),
            action_buttons
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        service_info = service_info.push(status_row);

        let copy_button = CopyToClipboardButton::new(
            String::from("logfile path"),
            String::from(&service.logfile_path),
        );
        let logfile_row = row![
            container(text("LOGFILE PATH:").size(20)).padding(Padding::default().top(8)),
            copy_button
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        service_info = service_info.push(logfile_row);

        let top_section = Section::new(service_info.into()).height(Length::Shrink);
        view = view.push(top_section);

        let log_data_view = scrollable(text(log_data).width(Length::Fill));
        let log_section = Section::new(log_data_view.into());
        view = view.push(log_section);

        view.into()
    }
}
