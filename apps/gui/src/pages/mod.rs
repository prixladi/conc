use std::fmt::Display;

use app_config::AppConfig;
use daemon_client::Requester;
use iced::{Element, Theme};
use project_page::ProjectPage;
use projects_page::ProjectsPage;
use service_page::ServicePage;
use settings_page::SettingsPage;

use crate::message::Message;

mod project_page;
mod projects_page;
mod service_page;
mod settings_page;

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Projects,
    Project(String),
    Service(String, String),
    Settings,
}

impl Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Projects => f.write_str("Projects"),
            Page::Project(project) => f.write_str(&format!("Project - {}", project)),
            Page::Service(project, service) => {
                f.write_str(&format!("Service - {}/{}", project, service))
            }
            Page::Settings => f.write_str("Settings"),
        }
    }
}

pub trait PageView {
    fn title(&self) -> String {
        self.page().to_string()
    }
    fn page(&self) -> Page;
    fn refresh(&mut self, data: PageData) -> Result<(), String>;
    fn view(&self) -> Element<Message>;
}

#[derive(Clone)]
pub struct PageData {
    pub requester: Requester,
    pub theme: Theme,
    pub config: AppConfig,
}

pub fn get_page(page_transition: Page) -> Box<dyn PageView> {
    match page_transition {
        Page::Projects => Box::new(ProjectsPage::new()),
        Page::Project(project) => Box::new(ProjectPage::new(project)),
        Page::Service(project, service) => Box::new(ServicePage::new(project, service)),
        Page::Settings => Box::new(SettingsPage::new()),
    }
}
