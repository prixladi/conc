use std::error::Error;

use crossterm::event::{KeyCode, KeyEvent};
use daemon_client::{ProjectInfo, Requester};
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Span, widgets::Row};

use crate::interactive::{
    components::{ActiveTable, CommonBlock},
    Action, ActionResult,
};

use super::{Page, PageView};

#[derive(Debug)]
pub(super) struct ProjectsPage {
    projects: Vec<ProjectInfo>,
    table: ActiveTable,
}

impl ProjectsPage {
    pub(super) fn new() -> Self {
        let table = ActiveTable::new()
            .ad_header(("NAME", 50))
            .ad_header(("STATUS", 50));

        ProjectsPage {
            projects: vec![],
            table,
        }
    }
}

impl PageView for ProjectsPage {
    fn handle_key_event(&mut self, key_event: KeyEvent, requester: &Requester) -> ActionResult {
        let selected_project = self.table.selected().map(|i| self.projects[i].clone());

        match key_event.code {
            KeyCode::Char('s') => {
                selected_project.map(|project| requester.start_project(&project.name).unwrap());
                Ok(Action::None)
            }
            KeyCode::Char('d') => {
                selected_project.map(|project| requester.stop_project(&project.name).unwrap());
                Ok(Action::None)
            }
            KeyCode::Enter | KeyCode::Right => match selected_project {
                Some(project) => Ok(Action::GotoPage(Page::Project(project.name))),
                None => Ok(Action::None),
            },
            event => {
                let total_count = self.projects.len();
                self.table.handle_key_code(event, total_count);
                Ok(Action::None)
            }
        }
    }

    fn refresh(&mut self, requester: &Requester) -> Result<(), Box<dyn Error>> {
        self.projects = requester.get_projects_info()?;
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = CommonBlock::new(String::from("Projects"))
            .add_instruction(("Open project", "enter"))
            .add_instruction(("Start project", "s"))
            .add_instruction(("Stop project", "d"))
            .add_instruction(("Quit", "q"));

        let rows = self
            .projects
            .iter()
            .cloned()
            .map(|project| {
                let status: Span = format!(
                    "{}/{} services running",
                    project.running_service_count(),
                    project.service_count()
                )
                .into();

                let name: Span = project.name.clone().into();

                let mut row = Row::new(vec![name, status]);
                if project.all_services_running() && project.service_count() > 0 {
                    row = row.green();
                } else if project.any_service_running() {
                    row = row.blue();
                }

                row
            })
            .collect();

        self.table.render(rows, block.into(), area, buf);
    }
}
