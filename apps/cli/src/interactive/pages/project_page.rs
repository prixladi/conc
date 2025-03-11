use std::error::Error;

use crossterm::event::{KeyCode, KeyEvent};
use daemon_client::{ProjectInfo, Requester, ServiceStatus};
use project_settings::ProjectSettings;
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Span, widgets::Row};

use crate::{
    interactive::{
        components::{ActiveTable, CommonBlock},
        Action, ActionResult,
    },
    utils::start_time_to_age,
};

use super::{Page, PageView};

#[derive(Debug)]
pub(super) struct ProjectPage {
    project_name: String,
    project: Option<ProjectInfo>,
    table: ActiveTable,
}

impl ProjectPage {
    pub(super) fn new(project_name: String) -> Self {
        let table = ActiveTable::new()
            .ad_header(("NAME", 25))
            .ad_header(("STATUS", 25))
            .ad_header(("PID", 25))
            .ad_header(("AGE", 25));

        ProjectPage {
            project_name,
            project: None,
            table,
        }
    }
}

impl PageView for ProjectPage {
    fn handle_key_event(&mut self, key_event: KeyEvent, requester: &Requester) -> ActionResult {
        let selected_service = self.table.selected().and_then(|i| {
            self.project
                .clone()
                .map(|project| project.services[i].clone())
        });

        match key_event.code {
            KeyCode::Char('s') => {
                if let Some(service) = selected_service {
                    requester.start_service(&self.project_name, &service.name)?;
                }
                Ok(Action::None)
            }
            KeyCode::Char('d') => {
                if let Some(service) = selected_service {
                    requester.stop_service(&self.project_name, &service.name)?;
                }
                Ok(Action::None)
            }
            KeyCode::Char('r') => {
                if let Some(service) = selected_service {
                    requester.restart_service(&self.project_name, &service.name)?;
                }
                Ok(Action::None)
            }
            KeyCode::Char('o') => {
                let settings = requester.get_project_settings(&self.project_name)?;
                Ok(Action::OpenString(ProjectSettings::prettify_json(
                    &settings,
                )?))
            }
            KeyCode::Enter => {
                let action = selected_service
                    .map(|service| Action::OpenLogs(service.logfile_path))
                    .unwrap_or(Action::None);

                Ok(action)
            }
            KeyCode::Left | KeyCode::Char('h') => {
                Ok(Action::GotoPage(Page::Projects))
            }
            event => {
                let total_count = self
                    .project
                    .as_ref()
                    .map(|proj| proj.services.len())
                    .unwrap_or_default();

                self.table.handle_key_code(event, total_count);
                Ok(Action::None)
            }
        }
    }

    fn refresh(&mut self, requester: &Requester) -> Result<(), Box<dyn Error>> {
        self.project = Some(requester.get_project_info(&self.project_name)?);
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if self.project.is_none() {
            return;
        }
        let project = self.project.as_ref().unwrap();

        let block = CommonBlock::new(format!("Project: {}", project.name))
            .add_instruction(("Start service", "s"))
            .add_instruction(("Stop service", "d"))
            .add_instruction(("Restart service", "r"))
            .add_instruction(("Service logs", "enter"))
            .add_instruction(("Project settings", "o"))
            .add_instruction(("Back", "h"))
            .add_instruction(("Quit", "q"));

        let rows = project
            .services
            .iter()
            .cloned()
            .map(|service| {
                let status: Span = service.status.to_string().into();
                let name: Span = service.name.clone().into();
                let pid: Span = service.pid.to_string().into();
                let age: Span = match service.status {
                    ServiceStatus::RUNNING => start_time_to_age(service.start_time),
                    _ => String::new(),
                }
                .into();

                let mut row = Row::new(vec![name, status, pid, age]);
                if service.status == ServiceStatus::RUNNING {
                    row = row.green();
                }

                row
            })
            .collect();

        self.table.render(rows, block.into(), area, buf);
    }
}
