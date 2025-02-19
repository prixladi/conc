use daemon_client::ProjectInfo;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};
use project_settings::ProjectSettings;

use crate::components::{
    CopyToClipboardButton, InfoTable, PageTitle, ProjectActionButtons, Section,
    ServiceActionButtons,
};
use crate::message::Message;

use super::{Page, PageData, PageView};

pub struct ProjectPage {
    project_name: String,
    project: Option<(ProjectInfo, String)>,
}

impl ProjectPage {
    pub fn new(project_name: String) -> Self {
        Self {
            project_name,
            project: None,
        }
    }
}

impl PageView for ProjectPage {
    fn page(&self) -> Page {
        Page::Project(self.project_name.clone())
    }

    fn refresh(&mut self, data: PageData) -> Result<(), String> {
        let result = data
            .requester
            .get_project_info(&self.project_name)
            .and_then(|project| {
                let settings = data.requester.get_project_settings(&self.project_name)?;
                Ok((project, settings))
            });

        match result {
            Ok(project) => {
                self.project = Some(project);
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn view(&self) -> Element<Message> {
        let mut view = column![];
        if self.project.is_none() {
            return view.into();
        };
        let (project, settings) = self.project.as_ref().unwrap();

        let mut names = vec![];
        let mut statuses = vec![];
        let mut actions = vec![];
        for service in project.services.iter() {
            names.push(service.name.clone());
            statuses.push(service.status.to_string());
            actions.push(ServiceActionButtons::new(&project.name, service).into());
        }

        let pretty_settings = ProjectSettings::prettify_json(settings).unwrap_or_default();
        let json_view = scrollable(text(pretty_settings.clone()).width(Length::Fill));

        let copy_button =
            CopyToClipboardButton::new(String::from("project settings"), pretty_settings);
        let action_buttons = ProjectActionButtons::new(project);
        let button_line = row![action_buttons, copy_button].spacing(10).into();

        let title = PageTitle::new(self.title())
            .additional_content(button_line)
            .into();
        let name_to_message = |service: &str| {
            Message::GotoPage(Page::Service(project.name.clone(), service.to_string()))
        };
        let table = InfoTable::new(title, names, statuses, actions, name_to_message);

        let project_view = container(table).height(Length::Fill).width(Length::Fill);

        view = view.push(Section::new(project_view.into()));
        view = view.push(Section::new(json_view.into()));

        view.into()
    }
}
