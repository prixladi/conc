use daemon_client::{ProjectInfo, Requester, ServiceStatus};
use iced::widget::container;
use iced::{Element, Length};

use crate::components::{InfoTable, ProjectActions, Section};
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
        let mut names = vec![];
        let mut statuses = vec![];
        let mut actions = vec![];
        for project in &self.projects {
            let services_count = project.services.len();
            let running_services_count = project
                .services
                .iter()
                .filter(|service| service.status == ServiceStatus::RUNNING)
                .count();

            names.push(project.name.clone());
            statuses.push(format!(
                "{}/{} services running",
                running_services_count, services_count
            ));
            actions.push(ProjectActions::new(project).render());
        }

        let table = InfoTable::new(self.title(), names, statuses, actions);

        let view = container(table.render())
            .height(Length::Fill)
            .width(Length::Fill);

        Section::new().render(view)
    }
}
