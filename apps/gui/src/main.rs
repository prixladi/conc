use std::time::Duration;

use chrono::{DateTime, Local};
use components::{Menu, StatusErrorBar, StatusInfoBar};
use daemon_client::{Requester, SocketClient};
use iced::widget::{column, row};
use iced::{Element, Task, Theme};
use message::Message;
use pages::{get_page, Page, PageView};
use tokio::time::sleep;

mod components;
mod message;
mod pages;

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .theme(|_| App::theme())
        .run_with(App::new)
}

struct App {
    requester: Requester,
    last_refresh_at: DateTime<Local>,
    error: Option<String>,
    project_names: Vec<String>,
    page_view: Box<dyn PageView>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let socket_client = SocketClient::new("../daemon/run/conc.sock");
        let requester = Requester::new(socket_client);

        let app = Self {
            page_view: get_page(requester.clone(), Page::Projects),
            requester,
            last_refresh_at: Local::now(),
            project_names: vec![],
            error: None,
        };

        (app, Task::done(Message::RefreshLoop))
    }
}

impl App {
    fn theme() -> Theme {
        Theme::Ferra
    }

    fn title(&self) -> String {
        format!("ConcG | {}", self.page_view.title())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let refresh = match &message {
            Message::RefreshLoop => true,
            Message::StartProject { name } => {
                self.requester.start_project(name).unwrap();
                true
            }
            Message::RestartProject { name } => {
                self.requester.restart_project(name).unwrap();
                true
            }
            Message::StopProject { name } => {
                self.requester.stop_project(name).unwrap();
                true
            }
            Message::GotoPage { page } => {
                self.page_view = get_page(self.requester.clone(), page.clone());
                true
            }
        };

        if refresh {
            match self.requester.get_project_names() {
                Ok(info) => {
                    self.project_names = info.values;
                    self.error = None;
                }
                Err(err) => self.error = Some(err.to_string()),
            }
            if let Err(err) = self.page_view.refresh() {
                self.error = Some(err);
            }

            self.last_refresh_at = Local::now();
        };

        if let Message::RefreshLoop = message {
            return Task::perform(sleep(Duration::from_secs(1)), |_| Message::RefreshLoop);
        };
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let view = self.page_view.view();

        let menu = Menu::new(self.project_names.clone(), self.page_view.page());
        let body = row![menu.render(), view];

        let info_bar = StatusInfoBar::new(
            self.last_refresh_at,
            self.requester.client().socket_path.clone(),
        );
        let error_bar = StatusErrorBar::new(self.error.clone());

        column![error_bar.render(), body, info_bar.render(),].into()
    }
}
