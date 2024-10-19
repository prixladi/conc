use std::time::Duration;

use app_config::CliConfig;
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
mod utils;

pub fn main() -> iced::Result {
    let config = CliConfig::new().unwrap();

    iced::application(App::title, App::update, App::view)
        .theme(|_| App::theme())
        .run_with(|| App::new(config))
}

struct App {
    requester: Requester,
    project_names: Vec<String>,
    page_view: Box<dyn PageView>,

    last_action_at: DateTime<Local>,
    last_action_result: Result<String, String>,
    refresh_loop_error: Option<String>,
}

impl App {
    fn new(config: CliConfig) -> (Self, Task<Message>) {
        let socket_client = SocketClient::new(&config.daemon_socket_path);
        let requester = Requester::new(socket_client);

        let app = Self {
            page_view: get_page(requester.clone(), Page::Projects),
            requester,
            last_action_at: Local::now(),
            project_names: vec![],
            last_action_result: Ok(String::from("Started")),
            refresh_loop_error: None,
        };

        (app, Task::done(Message::RefreshLoop))
    }
}

impl App {
    fn theme() -> Theme {
        Theme::Ferra
    }

    fn title(&self) -> String {
        format!("Conc | {}", self.page_view.title())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let res: Result<(String, bool), String> = match &message {
            Message::RefreshLoop => Ok((String::new(), true)),
            Message::GotoPage { page } => {
                self.page_view = get_page(self.requester.clone(), page.clone());
                Ok((format!("Navigated to the page '{}'", page), true))
            }
            Message::StartProject { name } => self
                .requester
                .start_project(name)
                .map(|_| (format!("Started project '{}'", name), true))
                .map_err(|err| format!("Unable to start project '{}': {}", name, err)),
            Message::RestartProject { name } => self
                .requester
                .restart_project(name)
                .map(|_| (format!("Restarted project '{}'", name), true))
                .map_err(|err| format!("Unable to restart project '{}': {}", name, err)),
            Message::StopProject { name } => self
                .requester
                .stop_project(name)
                .map(|_| (format!("Stopped project '{}'", name), true))
                .map_err(|err| format!("Unable to stop project '{}': {}", name, err)),
            Message::StartService { project, name } => self
                .requester
                .start_service(project, name)
                .map(|_| (format!("Started service '{}/{}'", project, name), true))
                .map_err(|err| format!("Unable to start service '{}/{}': {}", project, name, err)),
            Message::RestartService { project, name } => self
                .requester
                .restart_service(project, name)
                .map(|_| (format!("Restarted service '{}/{}'", project, name), true))
                .map_err(|err| {
                    format!("Unable to restart service '{}/{}': {}", project, name, err)
                }),
            Message::StopService { project, name } => self
                .requester
                .stop_service(project, name)
                .map(|_| (format!("Restarted service '{}/{}'", project, name), true))
                .map_err(|err| {
                    format!("Unable to restart service '{}/{}': {}", project, name, err)
                }),
        };

        if let Ok((_, true)) = res {
            match self.requester.get_project_names() {
                Ok(info) => {
                    self.project_names = info.values;
                    self.refresh_loop_error = None;
                }
                Err(err) => {
                    self.refresh_loop_error = Some(err.to_string());
                    self.last_action_at = Local::now();
                }
            }
            if let Err(err) = self.page_view.refresh() {
                self.refresh_loop_error = Some(err);
                self.last_action_at = Local::now();
            }
        };

        if let Message::RefreshLoop = message {
            return Task::perform(sleep(Duration::from_secs(1)), |_| Message::RefreshLoop);
        };

        self.last_action_result = res.map(|(status, _)| status);
        self.last_action_at = Local::now();

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let view = self.page_view.view();

        let menu = Menu::new(self.project_names.clone(), self.page_view.page());
        let body = row![menu.render(), view];

        let info_bar = StatusInfoBar::new(self.requester.client().socket_path.clone());
        let status: Result<String, String> = if let Some(err) = &self.refresh_loop_error {
            Err(err.clone())
        } else {
            self.last_action_result.clone()
        };

        let error_bar = StatusErrorBar::new(self.last_action_at, status);

        column![error_bar.render(), body, info_bar.render()].into()
    }
}
