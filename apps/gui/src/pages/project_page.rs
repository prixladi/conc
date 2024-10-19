use daemon_client::{ProjectInfo, Requester};
use iced::widget::{center, column, container, text};
use iced::{alignment, Element, Length};

use crate::message::Message;

use super::{Page, PageView};

pub struct ProjectPage {
    requester: Requester,
    project_name: String,
    project: Option<ProjectInfo>,
}

impl ProjectPage {
    pub fn new(requester: Requester, project_name: String) -> Self {
        Self {
            requester,
            project_name,
            project: None,
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
        match self.requester.get_project_info(&self.project_name) {
            Ok(info) => {
                self.project = Some(info.value);
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn view(&self) -> Element<Message> {
        let content = center(
            column![text(self.project_name.to_string()).size(50)]
                .spacing(20)
                .padding(20)
                .max_width(600),
        );

        let view = container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center);

        view.into()
    }
}
