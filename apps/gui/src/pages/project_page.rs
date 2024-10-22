use daemon_client::{ProjectInfo, Requester};
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};
use project_settings::ProjectSettings;

use crate::components::{
    CopyToClipboardButton, InfoTable, PageTitle, ProjectActionButtons, Section,
    ServiceActionButtons,
};
use crate::message::Message;
use crate::utils::{prettify_json, service_status_stringify};

use super::{Page, PageData, PageView};

pub struct ProjectPage {
    requester: Requester,
    project_name: String,
    project: Option<(ProjectInfo, String)>,
}

impl ProjectPage {
    pub fn new(data: PageData, project_name: String) -> Self {
        Self {
            requester: data.requester,
            project_name,
            project: None,
        }
    }
}

impl PageView for ProjectPage {
    fn page(&self) -> Page {
        Page::Project(self.project_name.clone())
    }

    fn refresh(&mut self, _: PageData) -> Result<(), String> {
        let result = self
            .requester
            .get_project_info(&self.project_name)
            .and_then(|project| {
                let settings = self.requester.get_project_settings(&self.project_name)?;
                Ok((project.value, settings.value))
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
            statuses.push(service_status_stringify(&service.status));
            actions.push(ServiceActionButtons::new(&project.name, service).render());
        }

        let pretty_settings = prettify_json::<ProjectSettings>(settings).unwrap_or_default();
        let json_view = scrollable(text(pretty_settings.clone()).width(Length::Fill));
        let settings_section = Section::new(json_view.into()).render();

        let copy_button =
            CopyToClipboardButton::new(String::from("project settings"), pretty_settings).render();
        let action_buttons = ProjectActionButtons::new(project).render();
        let button_line = row![action_buttons, copy_button].spacing(10).into();

        let tile = PageTitle::new(self.title(), Some(button_line)).render();
        let name_to_message = |service: &str| {
            Message::GotoPage(Page::Service(project.name.clone(), service.to_string()))
        };
        let table = InfoTable::new(tile, names, statuses, actions, name_to_message);

        let project_view = container(table.render())
            .height(Length::Fill)
            .width(Length::Fill);

        view = view.push(Section::new(project_view.into()).render());
        view = view.push(settings_section);

        view.into()
    }
}
