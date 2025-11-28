use std::{error::Error, time::Duration};

use app_config::AppConfig;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use daemon_client::Requester;
use external_command::{open_log_file, open_string_in_less};
use pages::{Page, PageContext, PageManager};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget, DefaultTerminal, Frame};
use tui_settings::{LogPreviewSettings, TuiSettings};

mod components;
mod external_command;
mod pages;
mod tui_settings;
mod keybind_utils;

pub fn interact(requester: Requester, config: AppConfig) -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    terminal.clear().unwrap();
    let res = App::new(requester, config).run(&mut terminal);
    ratatui::restore();
    res
}

enum Action {
    None,
    Exit,
    GotoPage(Page),
    OpenLogs(String),
    OpenString(String),
}

type ActionResult = Result<Action, Box<dyn Error>>;

struct App {
    requester: Requester,
    page_manager: PageManager,
    settings: TuiSettings,
    config: AppConfig,
}

impl App {
    fn new(requester: Requester, config: AppConfig) -> Self {
        App {
            requester,
            page_manager: PageManager::new(Page::Projects),
            settings: TuiSettings {
                log_preview: LogPreviewSettings::On,
            },
            config,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        loop {
            let context = self.create_page_context();
            self.page_manager.view().update(context)?;
            terminal.draw(|frame| self.draw(frame))?;

            match self.handle_events()? {
                Action::Exit => break Ok(()),
                Action::GotoPage(page) => {
                    self.page_manager.goto_page(page);
                }
                Action::OpenLogs(path) => {
                    open_log_file(terminal, &self.config.log_view_command, path)?;
                }
                Action::OpenString(str) => {
                    open_string_in_less(terminal, str)?;
                }
                Action::None => {}
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let context = self.create_page_context();
        let cursor_position = self.page_manager.view().cursor_position(area, context);
        if let Some(cp) = cursor_position {
            frame.set_cursor_position(cp)
        }

        frame.render_widget(self, area);
    }

    fn handle_events(&mut self) -> ActionResult {
        if event::poll(Duration::from_secs(1))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => Ok(Action::None),
            }
        } else {
            Ok(Action::None)
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> ActionResult {
        if self.page_manager.is_in_raw_mode() {
            return self.handle_key_event_page(key_event);
        }

        match self.handle_key_event_global(key_event) {
            Some(res) => res,
            None => self.handle_key_event_page(key_event),
        }
    }

    fn handle_key_event_page(&mut self, key_event: KeyEvent) -> ActionResult {
        let context = self.create_page_context();
        self.page_manager
            .view()
            .handle_key_event(key_event, context)
    }

    fn handle_key_event_global(&mut self, key_event: KeyEvent) -> Option<ActionResult> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc if key_event.kind == KeyEventKind::Press => {
                Some(Ok(Action::Exit))
            }
            KeyCode::Tab => {
                let current_page = self.page_manager.current_page().clone();

                match current_page {
                    Page::Keybinds(_) => None,
                    _ => Some(Ok(Action::GotoPage(Page::Keybinds(Box::new(current_page))))),
                }
            }
            KeyCode::Char('i') => {
                self.settings.log_preview = match self.settings.log_preview {
                    LogPreviewSettings::On => LogPreviewSettings::Fit,
                    LogPreviewSettings::Off => LogPreviewSettings::On,
                    LogPreviewSettings::Fit => LogPreviewSettings::Off,
                };

                Some(Ok(Action::None))
            }
            _ => None,
        }
    }

    fn create_page_context(&self) -> PageContext {
        PageContext {
            requester: self.requester.clone(),
            settings: self.settings.clone(),
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let context = self.create_page_context();
        self.page_manager.view().render(area, buf, context);
    }
}
