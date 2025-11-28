use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Stylize},
    widgets::{Cell, Row, StatefulWidget, Table, TableState},
};

use crate::interactive::{components::CommonBlock, Action, ActionResult};

use super::{Page, PageContext, PageView};

#[derive(Debug)]
pub(super) struct KeybindsPage {
    for_page: Page,
    state: TableState,
}

impl KeybindsPage {
    pub(super) fn new(for_page: Page) -> Self {
        Self {
            for_page,
            state: TableState::new(),
        }
    }
}

impl PageView for KeybindsPage {
    fn handle_key_event(&mut self, key_event: KeyEvent, _: PageContext) -> ActionResult {
        match key_event.code {
            KeyCode::Tab | KeyCode::Esc | KeyCode::Char('q') => {
                Ok(Action::GotoPage(self.for_page.clone()))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let offset = self.state.offset() + 1;
                self.state = self.state.clone().with_offset(offset);

                Ok(Action::None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let offset = if self.state.offset() == 0 {
                    0
                } else {
                    self.state.offset() - 1
                };
                self.state = self.state.clone().with_offset(offset);
                Ok(Action::None)
            }
            _ => Ok(Action::None),
        }
    }

    fn on_mount(&mut self) {
        self.state = TableState::new();
    }

    fn is_in_raw_mode(&self) -> bool {
        true
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, _: PageContext) {
        let (title, binds) = match &self.for_page {
            Page::Projects => (String::from("PROJECTS"), get_projects_page_keybinds()),
            Page::Project(project) => (
                format!("PROJECT '{}'", project),
                get_project_page_keybinds(),
            ),
            Page::Keybinds(_) => (String::from("keybinds"), get_keybinds_keybinds()),
        };

        let block = CommonBlock::new(format!("Keybind for page: {}", title))
            .set_border_color(Color::Green)
            .add_instruction(("Return", "tab"));

        let rows = binds.iter().map(|(key, bind)| {
            Row::new(vec![
                Cell::from(format!(" {}", bind.join(", ")).magenta().bold()),
                Cell::from(format!(" {}", *key)),
            ])
        });

        let header = Row::new(vec![Cell::from(" Bind".bold()), Cell::from(" Key".bold())]);
        let widths = vec![Constraint::Length(20), Constraint::Fill(1)];
        let table = Table::new(rows, widths).header(header).block(block.into());
        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}

fn get_projects_page_keybinds() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Search projects", vec!["/"]),
        ("Start project", vec!["s"]),
        ("Stop project", vec!["d"]),
        ("Restart project", vec!["r"]),
        ("Clear project logs", vec!["ctrl+alt+l"]),
        ("Open project", vec!["enter", "k", "right"]),
        ("Next project", vec!["j", "down"]),
        ("Previous project", vec!["l", "up"]),
        ("Change log preview mode", vec!["i"]),
        ("Quit app", vec!["q", "esc"]),
        ("Show keybinds", vec!["tab"]),
    ]
}

fn get_project_page_keybinds() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Search services", vec!["/"]),
        ("Start service", vec!["s"]),
        ("Stop service", vec!["d"]),
        ("Restart service", vec!["r"]),
        ("Clear service logs", vec!["ctrl+alt+l"]),
        ("Next service", vec!["j", "down"]),
        ("Previous service", vec!["l", "up"]),
        ("Open logs", vec!["enter"]),
        ("Go back to projects", vec!["j", "left"]),
        ("Change log preview mode", vec!["i"]),
        ("Open project settings", vec!["o"]),
        ("Quit app", vec!["q", "esc"]),
        ("Start project", vec!["S"]),
        ("Stop project", vec!["D"]),
        ("Restart project", vec!["R"]),
        ("Show keybinds", vec!["tab"]),
    ]
}

fn get_keybinds_keybinds() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![("Go back", vec!["q", "esc", "tab"])]
}
