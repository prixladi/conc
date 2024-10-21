use daemon_client::{ProjectInfo, Requester};
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length};
use project_settings::ProjectSettings;

use crate::components::{InfoTable, PageTitle, ProjectActions, Section, ServiceActions};
use crate::message::Message;
use crate::utils::{prettify_json, service_status_stringify};

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
        let mut names = vec![];
        let mut statuses = vec![];
        let mut actions = vec![];

        let mut view = column![];

        if let Some(project) = &self.project {
            for service in project.services.iter() {
                names.push(service.name.clone());
                statuses.push(service_status_stringify(&service.status));
                actions.push(ServiceActions::new(&project.name, service).render());
            }

            let action_buttons = ProjectActions::new(project).render();

            let tile = PageTitle::new(self.title()).render(Some(action_buttons));
            let table = InfoTable::new(names, statuses, actions);
            let name_to_message = |service: &str| {
                Message::GotoPage(Page::Service(project.name.clone(), service.to_string()))
            };
            let project_view = container(table.render(name_to_message, tile))
                .height(Length::Fill)
                .width(Length::Fill);

            view = view.push(Section::new().render(project_view));
        }

        if let Some(settings) = &self.project_settings {
            let pretty_settings = prettify_json::<ProjectSettings>(settings).unwrap_or_default();
            let json_view = scrollable(text(pretty_settings).width(Length::Fill));
            let settings_section = Section::new().render(json_view);
            view = view.push(settings_section);
        }

        view.into()
    }
}
