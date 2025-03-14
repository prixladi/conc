use std::{error::Error, vec};

use crossterm::event::{KeyCode, KeyEvent};
use daemon_client::{ProjectInfo, Requester, ServiceInfo, ServiceStatus};
use project_settings::ProjectSettings;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Stylize},
    text::Span,
    widgets::Row,
};

use crate::{
    interactive::{
        components::{ActiveTable, CommonBlock, Input},
        Action, ActionResult,
    },
    utils::start_time_to_age,
};

use super::{Page, PageView};

#[derive(Debug, PartialEq)]
enum Focus {
    Table,
    Search,
}

#[derive(Debug)]
pub(super) struct ProjectPage {
    project_name: String,
    project: Option<ProjectInfo>,

    focus: Focus,
    table: ActiveTable,
    search: Input,
}

impl ProjectPage {
    pub(super) fn new(project_name: String) -> Self {
        let table = ActiveTable::new()
            .ad_header(("NAME", 25))
            .ad_header(("STATUS", 25))
            .ad_header(("PID", 25))
            .ad_header(("AGE", 25));

        let input = Input::new();

        ProjectPage {
            project_name,
            project: None,
            table,
            focus: Focus::Table,
            search: input,
        }
    }
}

impl PageView for ProjectPage {
    fn refresh(&mut self, requester: &Requester) -> Result<(), Box<dyn Error>> {
        self.project = Some(requester.get_project_info(&self.project_name)?);
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, requester: &Requester) -> ActionResult {
        match self.focus {
            Focus::Table => self.handle_key_event_table(key_event, requester),
            Focus::Search => self.handle_key_event_search(key_event),
        }
    }

    fn cursor_position(&self, area: Rect) -> Option<Position> {
        match self.focus {
            Focus::Table => None,
            Focus::Search => {
                let [search_area, _] = self.get_full_layout(area);
                Some(Position::new(
                    search_area.x + self.search.len() as u16 + 1,
                    search_area.y + 1,
                ))
            }
        }
    }

    fn on_mount(&mut self) {
        self.focus = Focus::Table;
        self.search.clear();
    }

    fn is_in_raw_mode(&self) -> bool {
        self.focus == Focus::Search
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if self.search.is_empty() && self.focus == Focus::Table {
            self.render_table(area, buf);
            return;
        }

        let [search_area, table_area] = self.get_full_layout(area);
        self.render_search(search_area, buf);
        self.render_table(table_area, buf);
    }
}

impl ProjectPage {
    fn handle_key_event_table(
        &mut self,
        key_event: KeyEvent,
        requester: &Requester,
    ) -> ActionResult {
        let selected_service = self.get_selected_service();

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
            KeyCode::Left | KeyCode::Char('h') => Ok(Action::GotoPage(Page::Projects)),
            KeyCode::Char('/') => {
                self.focus = Focus::Search;
                Ok(Action::None)
            }
            event => {
                let service_count = self.get_filtered_services().len();
                self.table.handle_key_code(event, service_count);
                Ok(Action::None)
            }
        }
    }

    fn handle_key_event_search(&mut self, key_event: KeyEvent) -> ActionResult {
        match key_event.code {
            KeyCode::Esc => {
                self.focus = Focus::Table;
                self.search.clear();
            }
            KeyCode::Char('/') | KeyCode::Enter => self.focus = Focus::Table,
            code => self.search.handle_key_code(code),
        }

        Ok(Action::None)
    }

    fn get_filtered_services(&self) -> Vec<ServiceInfo> {
        match &self.project {
            Some(project) => project
                .services
                .iter()
                .filter(|service| {
                    self.search.is_empty() || service.name.contains(&self.search.value())
                })
                .cloned()
                .collect(),
            None => vec![],
        }
    }

    fn get_selected_service(&self) -> Option<ServiceInfo> {
        self.table.selected().and_then(|i| {
            let services = self.get_filtered_services();
            if services.is_empty() {
                None
            } else if i > services.len() {
                Some(services[0].clone())
            } else {
                Some(services[i].clone())
            }
        })
    }

    fn get_full_layout(&self, area: Rect) -> [Rect; 2] {
        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]);
        vertical.areas(area)
    }

    fn render_search(&mut self, area: Rect, buf: &mut Buffer) {
        let block = CommonBlock::new(String::from("Search"))
            .set_border_color(Color::LightRed)
            .add_instruction(("Search", "/"))
            .add_instruction(("Clear", "escape"));

        self.search.render(block.into(), area, buf);
    }

    fn render_table(&mut self, area: Rect, buf: &mut Buffer) {
        if self.project.is_none() {
            return;
        }
        let project = self.project.as_ref().unwrap();

        let block = CommonBlock::new(format!("Project: {}", project.name))
            .set_border_color(Color::LightBlue)
            .add_instruction(("Search", "/"))
            .add_instruction(("Start", "s"))
            .add_instruction(("Stop", "d"))
            .add_instruction(("Restart", "r"))
            .add_instruction(("Logs", "enter"))
            .add_instruction(("Project settings", "o"))
            .add_instruction(("Back", "h"))
            .add_instruction(("Quit", "q"));

        let rows = self
            .get_filtered_services()
            .iter()
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
