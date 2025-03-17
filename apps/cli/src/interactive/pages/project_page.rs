use std::{error::Error, vec};

use std::cmp::{max, min};

use ansi_to_tui::IntoText;
use crossterm::event::{KeyCode, KeyEvent};
use daemon_client::{ProjectInfo, Requester, ServiceInfo, ServiceStatus};
use project_settings::ProjectSettings;
use ratatui::text::Text;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Stylize},
    text::Span,
    widgets::{Paragraph, Row, Widget},
};

use crate::interactive::tui_settings::{LogPreviewSettings, TuiSettings};
use crate::{
    interactive::{
        components::{ActiveTable, CommonBlock, Input},
        Action, ActionResult,
    },
    utils::{read_last_n_lines_from_file, start_time_to_age},
};

use super::{Page, PageContext, PageView};

#[derive(Debug, PartialEq)]
enum Focus {
    Table,
    Search,
}

#[derive(Debug)]
pub(super) struct ProjectPage {
    project_name: String,
    project: Option<ProjectInfo>,
    logs: Vec<String>,

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

        Self {
            project_name,
            project: None,
            table,
            focus: Focus::Table,
            search: input,
            logs: vec![],
        }
    }
}

impl PageView for ProjectPage {
    fn update(&mut self, context: PageContext) -> Result<(), Box<dyn Error>> {
        let project = context.requester.get_project_info(&self.project_name)?;
        self.project = Some(project);

        if let Some(selected_service) = self.get_selected_service() {
            if context.settings.log_preview != LogPreviewSettings::Off {
                let lines = read_last_n_lines_from_file(&selected_service.logfile_path)?;
                self.logs = lines;
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, context: PageContext) -> ActionResult {
        match self.focus {
            Focus::Table => self.handle_key_event_table(key_event, &context.requester),
            Focus::Search => self.handle_key_event_search(key_event),
        }
    }

    fn cursor_position(&self, area: Rect, context: PageContext) -> Option<Position> {
        match self.focus {
            Focus::Table => None,
            Focus::Search => {
                let layout = PageLayout::from(self, area, &context.settings);
                layout.search_area.map(|search_area| {
                    Position::new(
                        search_area.x + self.search.len() as u16 + 1,
                        search_area.y + 1,
                    )
                })
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

    fn render(&mut self, area: Rect, buf: &mut Buffer, context: PageContext) {
        let layout = PageLayout::from(self, area, &context.settings);

        if let Some(search_area) = layout.search_area {
            self.render_search(search_area, buf);
        }
        self.render_table(layout.table_area, buf, context);

        if let Some(logs_area) = layout.logs_area {
            self.render_logs(logs_area, buf);
        }
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
            KeyCode::Char('S') => {
                requester.start_project(&self.project_name)?;
                Ok(Action::None)
            }
            KeyCode::Char('D') => {
                requester.stop_project(&self.project_name)?;
                Ok(Action::None)
            }
            KeyCode::Char('R') => {
                requester.restart_project(&self.project_name)?;
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
        let selected = self.table.selected().unwrap_or_default();
        let services = self.get_filtered_services();

        if services.is_empty() {
            None
        } else if selected == services.len() {
            Some(services[0].clone())
        } else if selected >= services.len() {
            services.last().cloned()
        } else {
            Some(services[selected].clone())
        }
    }

    fn render_search(&mut self, area: Rect, buf: &mut Buffer) {
        let block = CommonBlock::new(String::from("Search"))
            .set_border_color(Color::LightRed)
            .add_instruction(("Search", "/"))
            .add_instruction(("Clear", "escape"));

        self.search.render(block.into(), area, buf);
    }

    fn render_table(&mut self, area: Rect, buf: &mut Buffer, context: PageContext) {
        if self.project.is_none() {
            return;
        }
        let project = self.project.as_ref().unwrap();

        let title = format!(
            "Project: {} ({} services)",
            project.name,
            project.service_count()
        );
        let block = CommonBlock::new(title)
            .add_top_info(context.settings.to_info())
            .set_border_color(Color::LightBlue)
            .add_instruction(("Show keybinds", "tab"))
            .add_instruction(("Start", "s"))
            .add_instruction(("Stop", "d"))
            .add_instruction(("Logs", "enter"));

        let rows = self
            .get_filtered_services()
            .iter()
            .enumerate()
            .map(|(i, service)| {
                let status: Span = service.status.to_string().into();
                let name: Span = format!("{}. {}", i + 1, service.name).into();
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

    fn render_logs(&mut self, area: Rect, buf: &mut Buffer) {
        let block = CommonBlock::new(String::from("Logs")).set_border_color(Color::LightCyan);

        let content = if area.height < 3 {
            String::new()
        } else {
            let max_line_count = area.height - 2;
            self.logs
                .iter()
                .take(max_line_count as usize)
                .rev()
                .fold(String::new(), |a, b| format!("{} {}\n", a, b))
        };

        let text = content.clone().into_text().unwrap_or(Text::from(content));
        let input = Paragraph::new(text).block(block.into());
        input.render(area, buf);
    }
}

struct PageLayout {
    search_area: Option<Rect>,
    table_area: Rect,
    logs_area: Option<Rect>,
}

static TABLE_OVERHEAD: u16 = 3;
static MAX_FORCED_TABLE_LINE_COUNT: u16 = TABLE_OVERHEAD + 6;
static MIN_FORCED_TABLE_LINE_COUNT: u16 = TABLE_OVERHEAD + 1;
static MIN_LOGS_HEIGHT: u16 = 5;
static SEARCH_BAR_HEIGHT: u16 = 3;

impl PageLayout {
    fn from(page: &ProjectPage, area: Rect, settings: &TuiSettings) -> Self {
        let service_count = page
            .project
            .clone()
            .map(|p| p.service_count())
            .unwrap_or_default() as u16;

        let max_table_line_count = max(TABLE_OVERHEAD + service_count, MIN_FORCED_TABLE_LINE_COUNT);

        let show_search = !page.search.is_empty() || page.focus == Focus::Search;

        let table_line_display_count = match settings.log_preview {
            LogPreviewSettings::On => min(MAX_FORCED_TABLE_LINE_COUNT, max_table_line_count),
            LogPreviewSettings::Off | LogPreviewSettings::Fit => max_table_line_count,
        };

        let show_logs = match settings.log_preview {
            LogPreviewSettings::Off => false,
            LogPreviewSettings::On => area.height > MIN_LOGS_HEIGHT + table_line_display_count,
            LogPreviewSettings::Fit => area.height > (max_table_line_count + MIN_LOGS_HEIGHT),
        };

        if !show_search {
            if !show_logs {
                return PageLayout {
                    search_area: None,
                    table_area: area,
                    logs_area: None,
                };
            }

            let vertical = Layout::vertical([
                Constraint::Max(table_line_display_count),
                Constraint::Fill(1),
            ]);
            let [table, logs] = vertical.areas(area);

            return PageLayout {
                search_area: None,
                table_area: table,
                logs_area: Some(logs),
            };
        }

        if !show_logs {
            let vertical =
                Layout::vertical([Constraint::Length(SEARCH_BAR_HEIGHT), Constraint::Fill(1)]);
            let [search, table] = vertical.areas(area);

            return PageLayout {
                search_area: Some(search),
                table_area: table,
                logs_area: None,
            };
        }

        let vertical = Layout::vertical([
            Constraint::Length(SEARCH_BAR_HEIGHT),
            Constraint::Max(table_line_display_count - SEARCH_BAR_HEIGHT),
            Constraint::Fill(1),
        ]);
        let [search, table, logs] = vertical.areas(area);

        PageLayout {
            search_area: Some(search),
            table_area: table,
            logs_area: Some(logs),
        }
    }
}
