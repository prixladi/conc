use std::{collections::HashMap, error::Error, hash::Hash, mem};

use crossterm::event::KeyEvent;
use daemon_client::Requester;
use project_page::ProjectPage;
use projects_page::ProjectsPage;
use ratatui::{buffer::Buffer, layout::Rect};

use super::ActionResult;

mod project_page;
mod projects_page;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Page {
    Projects,
    Project(String),
}

pub trait PageView {
    fn handle_key_event(&mut self, key_event: KeyEvent, requester: &Requester) -> ActionResult;
    fn refresh(&mut self, requester: &Requester) -> Result<(), Box<dyn Error>>;
    fn render(&mut self, area: Rect, buf: &mut Buffer);
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
        self.cache.insert(self.page.clone(), view);

        self.page = page;
    }

    pub fn view(&mut self) -> &mut Box<dyn PageView> {
        &mut self.view
    }
}

fn create_page_view(page: Page) -> Box<dyn PageView> {
    match page {
        Page::Projects => Box::new(ProjectsPage::new()),
        Page::Project(project_name) => Box::new(ProjectPage::new(project_name)),
    }
}
