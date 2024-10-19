use daemon_client::{ProjectInfo, Requester, ServiceStatus};
use iced::widget::{button, center, column, container, row, scrollable, text, Column};
use iced::{alignment, Element, Length, Padding};

use crate::message::Message;

use super::{Page, PageView};

pub struct ProjectsPage {
    requester: Requester,
    projects: Vec<ProjectInfo>,
}

impl ProjectsPage {
    pub fn new(requester: Requester) -> Self {
        Self {
            projects: vec![],
            requester,
        }
    }
}

impl PageView for ProjectsPage {
    fn page(&self) -> Page {
        Page::Projects
    }

    fn title(&self) -> String {
        String::from("Projects")
    }

    fn refresh(&mut self) -> Result<(), String> {
        match self.requester.get_projects_info() {
            Ok(info) => {
                self.projects = info.values;
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn view(&self) -> Element<Message> {
        let mut names = column!["NAME"].spacing(10);
        let mut statuses = column!["STATUS"].spacing(10);
        let mut actions = column!["ACTIONS"].spacing(10);

        for project in &self.projects {
            let services_count = project.services.len();
            let running_services_count = project
                .services
                .iter()
                .filter(|service| service.status == ServiceStatus::RUNNING)
                .count();

            let name = cell(column![text(&project.name).size(18)]);
            let status = cell(column![text(format!(
                "{}/{} services running",
                running_services_count, services_count
            ))
            .size(18)]);

            let action_row = project_actions(project, services_count, running_services_count);

            names = names.push(name);
            statuses = statuses.push(status);
            actions = actions.push(cell(column![action_row]));
        }

        let rows = scrollable(row![names, statuses, actions].spacing(16));

        let content = center(
            column![text("Projects").size(50), rows]
                .spacing(20)
                .padding(10)
                .max_width(1200),
        );

        let view = container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center);

        view.into()
    }
}

fn project_actions<'a>(
    project: &ProjectInfo,
    services_count: usize,
    running_services_count: usize,
) -> Element<'a, Message> {
    let restart_button = button("Restart")
        .style(button::primary)
        .padding(Padding::default().top(7).bottom(2).left(4).right(4))
        .on_press(Message::RestartProject {
            name: project.name.clone(),
        });

    let mut start_button = button("Start")
        .style(button::success)
        .padding(Padding::default().top(7).bottom(2).left(4).right(4));
    if services_count > running_services_count {
        start_button = start_button.on_press(Message::StartProject {
            name: project.name.clone(),
        });
    };

    let mut stop_button = button("Stop")
        .style(button::danger)
        .padding(Padding::default().top(7).bottom(2).left(4).right(4));
    if running_services_count > 0 {
        stop_button = stop_button.on_press(Message::StopProject {
            name: project.name.clone(),
        });
    };

    row![start_button, stop_button, restart_button]
        .spacing(10)
        .into()
}

fn cell(content: Column<'_, Message>) -> Element<'_, Message> {
    container(content)
        .height(Length::Fill)
        .align_y(alignment::Vertical::Bottom)
        .height(30)
        .into()
}