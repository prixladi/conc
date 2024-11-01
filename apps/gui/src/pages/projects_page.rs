use daemon_client::ProjectInfo;
use iced::widget::container;
use iced::{Element, Length};

use crate::components::{InfoTable, PageTitle, ProjectActionButtons, Section};
use crate::message::Message;

use super::{Page, PageData, PageView};

pub struct ProjectsPage {
    projects: Vec<ProjectInfo>,
}

impl ProjectsPage {
    pub fn new() -> Self {
        Self { projects: vec![] }
    }
}

impl PageView for ProjectsPage {
    fn page(&self) -> Page {
        Page::Projects
    }

    fn refresh(&mut self, data: PageData) -> Result<(), String> {
        match data.requester.get_projects_info() {
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
            names.push(project.name.clone());
            statuses.push(format!(
                "{}/{} services running",
                project.running_service_count(),
                project.service_count()
            ));
            actions.push(ProjectActionButtons::new(project).into());
        }

        let title = PageTitle::new(self.title()).into();
        let name_to_message = |project: &str| Message::GotoPage(Page::Project(project.to_string()));
        let table = InfoTable::new(title, names, statuses, actions, name_to_message);

        let view = container(table).height(Length::Fill).width(Length::Fill);

        Section::new(view.into()).into()
    }
}
