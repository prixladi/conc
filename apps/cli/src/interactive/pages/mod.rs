use std::{collections::HashMap, error::Error, hash::Hash, mem};

use crossterm::event::KeyEvent;
use daemon_client::Requester;
use keybinds_page::KeybindsPage;
use project_page::ProjectPage;
use projects_page::ProjectsPage;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
};

use super::{tui_settings::TuiSettings, ActionResult};

mod keybinds_page;
mod project_page;
mod projects_page;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Page {
    Projects,
    Project(String),
    Keybinds(Box<Page>),
}

#[derive(Debug)]
pub struct PageContext {
    pub settings: TuiSettings,
    pub requester: Requester,
}

pub trait PageView {
    fn handle_key_event(&mut self, key_event: KeyEvent, context: PageContext) -> ActionResult;
    fn render(&mut self, area: Rect, buf: &mut Buffer, context: PageContext);
    fn update(&mut self, _: PageContext) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn on_mount(&mut self) {}
    fn cursor_position(&self, _: Rect, _: PageContext) -> Option<Position> {
        None
    }
    fn is_in_raw_mode(&self) -> bool {
        false
    }
}

pub struct PageManager {
    page: Page,
    view: Box<dyn PageView>,

    cache: HashMap<Page, Box<dyn PageView>>,
}

impl PageManager {
    pub fn new(page: Page) -> Self {
        Self {
            view: create_page_view(page.clone()),
            page,
            cache: HashMap::new(),
        }
    }

    pub fn goto_page(&mut self, page: Page) {
        let mut view = match self.cache.remove(&page) {
            Some(view) => view,
            None => create_page_view(page.clone()),
        };

        mem::swap(&mut view, &mut self.view);
        view.on_mount();
        self.cache.insert(self.page.clone(), view);

        self.page = page;
    }

    pub fn current_page(&self) -> &Page {
        &self.page
    }

    pub fn view(&mut self) -> &mut Box<dyn PageView> {
        &mut self.view
    }

    pub fn is_in_raw_mode(&self) -> bool {
        self.view.is_in_raw_mode()
    }
}

fn create_page_view(page: Page) -> Box<dyn PageView> {
    match page {
        Page::Projects => Box::new(ProjectsPage::new()),
        Page::Project(project_name) => Box::new(ProjectPage::new(project_name)),
        Page::Keybinds(prev_page) => Box::new(KeybindsPage::new(*prev_page)),
    }
}
