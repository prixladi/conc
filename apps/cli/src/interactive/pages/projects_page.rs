use std::error::Error;

use crossterm::event::{KeyCode, KeyEvent};
use daemon_client::{ProjectInfo, Requester};
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
pub(super) struct ProjectsPage {
    projects: Vec<ProjectInfo>,

    focus: Focus,
    table: ActiveTable,
    search: Input,
}

impl ProjectsPage {
    pub(super) fn new() -> Self {
        let table = ActiveTable::new()
            .ad_header(("NAME", 33))
            .ad_header(("STATUS", 33))
            .ad_header(("AGE", 33));

        let input = Input::new();

        ProjectsPage {
            projects: vec![],
            table,
            focus: Focus::Table,
            search: input,
        }
    }
}

impl PageView for ProjectsPage {
    fn refresh(&mut self, requester: &Requester) -> Result<(), Box<dyn Error>> {
        self.projects = requester.get_projects_info()?;
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

impl ProjectsPage {
    fn handle_key_event_table(
        &mut self,
        key_event: KeyEvent,
        requester: &Requester,
    ) -> ActionResult {
        let selected_project = self.get_selected_project();

        match key_event.code {
            KeyCode::Char('s') => {
                selected_project.map(|project| requester.start_project(&project.name).unwrap());
                Ok(Action::None)
            }
            KeyCode::Char('d') => {
                selected_project.map(|project| requester.stop_project(&project.name).unwrap());
                Ok(Action::None)
            }
            KeyCode::Char('r') => {
                selected_project.map(|project| requester.restart_project(&project.name).unwrap());
                Ok(Action::None)
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match selected_project {
                Some(project) => Ok(Action::GotoPage(Page::Project(project.name))),
                None => Ok(Action::None),
            },
            KeyCode::Char('/') => {
                self.focus = Focus::Search;
                Ok(Action::None)
            }
            event => {
                let project_count = self.get_filtered_projects().len();
                self.table.handle_key_code(event, project_count);
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

    fn get_filtered_projects(&self) -> Vec<ProjectInfo> {
        self.projects
            .iter()
            .filter(|project| self.search.is_empty() || project.name.contains(&self.search.value()))
            .cloned()
            .collect()
    }

    fn get_selected_project(&self) -> Option<ProjectInfo> {
        self.table.selected().and_then(|i| {
            let project = self.get_filtered_projects();
            if project.is_empty() {
                None
            } else if i > project.len() {
                Some(project[0].clone())
            } else {
                Some(project[i].clone())
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
        let block = CommonBlock::new(String::from("Projects"))
            .set_border_color(Color::LightYellow)
            .add_instruction(("Search", "/"))
            .add_instruction(("Open project", "l"))
            .add_instruction(("Start project", "s"))
            .add_instruction(("Stop project", "d"))
            .add_instruction(("Restart project", "r"))
            .add_instruction(("Quit", "q"));

        let rows = self
            .projects
            .iter()
            .filter(|project| self.search.is_empty() || project.name.contains(&self.search.value()))
            .map(|project| {
                let status: Span = format!(
                    "{}/{} services running",
                    project.running_service_count(),
                    project.service_count()
                )
                .into();
                let name: Span = project.name.clone().into();
                let age = match project.newest_running_service_started_at() {
                    Some(start_time) => start_time_to_age(start_time),
                    None => String::new(),
                }
                .into();

                let mut row = Row::new(vec![name, status, age]);
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
