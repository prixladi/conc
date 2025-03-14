use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug)]
pub struct Input {
    state: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            state: String::new(),
        }
    }

    pub fn handle_key_code(&mut self, code: KeyCode) {
        match code {
            KeyCode::Backspace => {
                self.state.pop();
            }
            KeyCode::Char(c) => {
                self.state.push(c);
            }
            _ => {}
        }
    }

    pub fn clear(&mut self) {
        self.state = String::new()
    }

    pub fn value(&self) -> String {
        self.state.clone()
    }

    pub fn len(&self) -> usize {
        self.state.len()
    }

    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    pub fn render(&mut self, block: Block<'_>, area: Rect, buf: &mut Buffer) {
        let input = Paragraph::new(self.state.clone()).block(block);
        Widget::render(input, area, buf);
    }
}
