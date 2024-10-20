use std::fmt::Display;

use daemon_client::Requester;
use iced::Element;
use project_page::ProjectPage;
use projects_page::ProjectsPage;
use service_page::ServicePage;

use crate::message::Message;

mod project_page;
mod projects_page;
mod service_page;

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Projects,
    Project(String),
    Service(String, String),
}

impl Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Projects => f.write_str("Projects"),
            Page::Project(project) => f.write_str(&format!("Project - {}", project)),
            Page::Service(project, service) => {
                f.write_str(&format!("Service - {}/{}", project, service))
            }
        }
    }
}

pub trait PageView {
    fn title(&self) -> String;
    fn page(&self) -> Page;
    fn refresh(&mut self) -> Result<(), String>;
    fn view(&self) -> Element<Message>;
}

pub fn get_page(requester: Requester, page_transition: Page) -> Box<dyn PageView> {
    match page_transition {
        Page::Projects => Box::new(ProjectsPage::new(requester)),
        Page::Project(project) => Box::new(ProjectPage::new(requester, project)),
        Page::Service(project, service) => Box::new(ServicePage::new(requester, project, service)),
    }
}
