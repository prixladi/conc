use std::{error::Error, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use daemon_client::Requester;
use external_command::{open_log_file_in_less, open_string_in_less};
use pages::{Page, PageManager};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget, DefaultTerminal, Frame};

mod components;
mod external_command;
mod pages;

pub fn interact(requester: Requester) -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    terminal.clear().unwrap();
    let res = App::new(requester).run(&mut terminal);
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
}

impl App {
    fn new(requester: Requester) -> Self {
        App {
            requester,
            page_manager: PageManager::new(Page::Projects),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        loop {
            self.page_manager.view().refresh(&self.requester)?;
            terminal.draw(|frame| self.draw(frame))?;

            match self.handle_events()? {
                Action::Exit => break Ok(()),
                Action::GotoPage(page) => {
                    self.page_manager.goto_page(page);
                }
                Action::OpenLogs(path) => {
                    open_log_file_in_less(terminal, path)?;
                }
                Action::OpenString(str) => {
                    open_string_in_less(terminal, str)?;
                }
                Action::None => {}
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(Action::Exit),
            _ => self
                .page_manager
                .view()
                .handle_key_event(key_event, &self.requester),
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.page_manager.view().render(area, buf);
    }
}
