use daemon_client::{ProjectInfo, Requester, ServiceInfo, ServiceStatus};
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced::{alignment, Element, Length, Padding};
use project_settings::ProjectSettings;

use crate::components::Section;
use crate::message::Message;
use crate::utils::prettify_json;

use super::{Page, PageView};

pub struct ProjectPage {
    requester: Requester,
    project_name: String,
    project: Option<ProjectInfo>,
    project_settings: Option<String>,
}

impl ProjectPage {
    pub fn new(requester: Requester, project_name: String) -> Self {
        Self {
            requester,
            project_name,
            project: None,
            project_settings: None,
        }
    }
}

impl PageView for ProjectPage {
    fn page(&self) -> Page {
        Page::Project(self.project_name.clone())
    }

    fn title(&self) -> String {
        format!("Project - {}", self.project_name)
    }

    fn refresh(&mut self) -> Result<(), String> {
        let result = self
            .requester
            .get_project_info(&self.project_name)
            .and_then(|project| {
                let settings = self.requester.get_project_settings(&self.project_name)?;
                Ok((project.value, settings.value))
            });

        match result {
            Ok((project, settings)) => {
                self.project = Some(project);
                self.project_settings = Some(settings);
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn view(&self) -> Element<Message> {
        let mut names = column!["NAME"].spacing(10);
        let mut statuses = column!["STATUS"].spacing(10);
        let mut actions = column!["ACTIONS"].spacing(10);

        if let Some(project) = &self.project {
            for service in project.services.iter() {
                let name = cell(text(&service.name).size(18));
                let status = cell(match service.status {
                    ServiceStatus::IDLE => "Idle",
                    ServiceStatus::RUNNING => "Running",
                    ServiceStatus::STOPPED => "Stopped",
                });
                let action_row = service_actions(&project.name, service);

                names = names.push(name);
                statuses = statuses.push(status);
                actions = actions.push(cell(action_row));
            }
        }

        let rows = scrollable(
            row![
                names,
                horizontal_space(),
                statuses,
                horizontal_space(),
                actions
            ]
            .spacing(8),
        );

        let content = column![
            text(format!("Project - {}", self.project_name)).size(30),
            rows
        ]
        .spacing(12)
        .padding(8);

        let project_view = container(content).height(Length::Fill).width(Length::Fill);
        let project_section = Section::new().render(project_view);

        let mut view = column![project_section];

        if let Some(settings) = &self.project_settings {
            let pretty_settings = prettify_json::<ProjectSettings>(settings).unwrap_or_default();
            let json_view = scrollable(text(pretty_settings).width(Length::Fill));
            let settings_view = container(json_view).padding(8);
            let settings_section = Section::new().render(settings_view);
            view = view.push(settings_section);
        }

        view.into()
    }
}

fn service_actions<'a>(project_name: &'a str, service: &ServiceInfo) -> Element<'a, Message> {
    let restart_button = button("Restart")
        .style(button::primary)
        .padding(Padding::default().top(7).bottom(2).left(4).right(4))
        .on_press(Message::RestartService {
            project: project_name.to_string(),
            name: service.name.clone(),
        });

    let mut start_button = button("Start")
        .style(button::success)
        .padding(Padding::default().top(7).bottom(2).left(4).right(4));
    if service.status != ServiceStatus::RUNNING {
        start_button = start_button.on_press(Message::StartService {
            project: project_name.to_string(),
            name: service.name.clone(),
        });
    };

    let mut stop_button = button("Stop")
        .style(button::danger)
        .padding(Padding::default().top(7).bottom(2).left(4).right(4));
    if service.status == ServiceStatus::RUNNING {
        stop_button = stop_button.on_press(Message::StopService {
            project: project_name.to_string(),
            name: service.name.clone(),
        });
    };

    row![start_button, stop_button, restart_button]
        .spacing(10)
        .into()
}

fn cell<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .height(Length::Fill)
        .align_y(alignment::Vertical::Bottom)
        .height(30)
        .into()
}
