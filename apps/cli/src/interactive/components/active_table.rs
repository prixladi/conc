use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{palette::tailwind::SLATE, Modifier, Style, Stylize},
    widgets::{Block, Cell, Row, StatefulWidget, Table, TableState},
};

#[derive(Debug)]
pub struct ActiveTable {
    state: TableState,
    headers: Vec<(&'static str, u16)>,
}

impl ActiveTable {}

impl ActiveTable {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select_first();

        Self {
            state,
            headers: vec![],
        }
    }

    pub fn ad_header(mut self, header: (&'static str, u16)) -> Self {
        self.headers.push(header);
        self
    }

    pub fn handle_key_code(&mut self, code: KeyCode, total_elements: usize) {
        if total_elements == 0 {
            return;
        };

        let current = self.selected().unwrap_or(0);

        match code {
            KeyCode::Down | KeyCode::Char('j') => match current >= total_elements - 1 {
                true => self.state.select_first(),
                false => self.state.select_next(),
            },
            KeyCode::Up | KeyCode::Char('k') => match current == 0 {
                true => self.state.select_last(),
                false => self.state.select_previous(),
            },
            _ => {}
        }
    }

    pub fn render<'a>(
        &mut self,
        rows: Vec<Row<'a>>,
        block: Block<'a>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let headers = self
            .headers
            .iter()
            .map(|(head, _)| Cell::from(head.bold()))
            .collect();
        let widths = self
            .headers
            .iter()
            .map(|(_, width)| Constraint::Percentage(*width));

        let table = Table::new(rows, widths)
            .header(headers)
            .block(block)
            .row_highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ");
        StatefulWidget::render(table, area, buf, &mut self.state);
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c900).add_modifier(Modifier::BOLD);
