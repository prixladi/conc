use daemon_client::{ProjectInfo, Requester, ServiceStatus};
use iced::widget::container;
use iced::{Element, Length};

use crate::components::{InfoTable, PageTitle, ProjectActionButtons, Section};
use crate::message::Message;

use super::{Page, PageData, PageView};

pub struct ProjectsPage {
    requester: Requester,
    projects: Vec<ProjectInfo>,
}

impl ProjectsPage {
    pub fn new(data: PageData) -> Self {
        Self {
            projects: vec![],
            requester: data.requester,
        }
    }
}

impl PageView for ProjectsPage {
    fn page(&self) -> Page {
        Page::Projects
    }

    fn refresh(&mut self, _: PageData) -> Result<(), String> {
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
            actions.push(ProjectActionButtons::new(project).render());
        }

        let title = PageTitle::new(self.title(), None).render();
        let name_to_message = |project: &str| Message::GotoPage(Page::Project(project.to_string()));
        let table = InfoTable::new(title, names, statuses, actions, name_to_message);

        let view = container(table.render())
            .height(Length::Fill)
            .width(Length::Fill);

        Section::new(view.into()).render()
    }
}
