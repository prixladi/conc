use std::time::Duration;

use app_config::AppConfig;
use chrono::{DateTime, Local};
use components::{Menu, StatusErrorBar, StatusInfoBar};
use daemon_client::{Requester, SocketClient};
use iced::widget::{column, row};
use iced::{Element, Task, Theme};
use message::Message;
use pages::{get_page, Page, PageData, PageView};
use tokio::time::sleep;

mod components;
mod message;
mod pages;
mod utils;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    let config = AppConfig::new().unwrap();
    iced::application(App::title, App::update, App::view)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .theme(App::theme)
        .run_with(|| App::new(config))
}

struct App {
    theme: Theme,
    config: AppConfig,

    requester: Requester,
    project_names: Vec<String>,
    page_view: Box<dyn PageView>,

    last_action_at: DateTime<Local>,
    last_action_result: Result<String, String>,
    refresh_loop_error: Option<String>,
}

impl App {
    fn new(config: AppConfig) -> (Self, Task<Message>) {
        let socket_client = SocketClient::new(&config.daemon_socket_path);
        let requester = Requester::new(socket_client);
        let theme = Theme::Dark;

        let page_view = get_page(
            Page::Projects,
            PageData {
                requester: requester.clone(),
                theme: theme.clone(),
                config: config.clone(),
            },
        );

        let app = Self {
            theme,
            config,

            requester,
            project_names: vec![],
            page_view,

            last_action_at: Local::now(),
            last_action_result: Ok(String::from("Started")),
            refresh_loop_error: None,
        };

        (app, Task::done(Message::Refresh { repeated: true }))
    }
}

impl App {
    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn title(&self) -> String {
        format!("Conc | {}", self.page_view.title())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match self.handle_message(&message) {
            Ok(task) => {
                let last_action_result = message.to_success_message();
                if !last_action_result.is_empty() {
                    self.last_action_result = Ok(last_action_result);
                    self.last_action_at = Local::now()
                }

                task
            }
            Err(err) => {
                self.last_action_result = Err(message.to_error_message(&err));
                self.last_action_at = Local::now();
                Task::none()
            }
        }
    }

    fn handle_message(&mut self, message: &Message) -> Result<Task<Message>, String> {
        match message {
            Message::Refresh { repeated } => {
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

                let page_data = PageData {
                    requester: self.requester.clone(),
                    theme: self.theme.clone(),
                    config: self.config.clone(),
                };
                if let Err(err) = self.page_view.refresh(page_data) {
                    self.refresh_loop_error = Some(err);
                    self.last_action_at = Local::now();
                }

                match repeated {
                    true => Ok(Task::perform(sleep(Duration::from_secs(1)), |_| {
                        Message::Refresh { repeated: true }
                    })),
                    false => Ok(Task::none()),
                }
            }

            Message::GotoPage(page) => {
                self.page_view = get_page(
                    page.clone(),
                    PageData {
                        requester: self.requester.clone(),
                        theme: self.theme.clone(),
                        config: self.config.clone(),
                    },
                );

                Ok(Task::done(Message::Refresh { repeated: false }))
            }

            Message::OpenUrl(url) => open::that(url)
                .map(|_| Task::none())
                .map_err(|err| err.to_string()),

            Message::ThemeChanged(theme) => {
                self.theme = theme.clone();
                Ok(Task::none())
            }

            Message::CopyToClipboard { name: _, data } => Ok(iced::clipboard::write(data.clone())),

            Message::StartProject { project_name } => self
                .requester
                .start_project(project_name)
                .map(|_| Task::done(Message::Refresh { repeated: false }))
                .map_err(|err| err.to_string()),

            Message::RestartProject { project_name } => self
                .requester
                .restart_project(project_name)
                .map(|_| Task::done(Message::Refresh { repeated: false }))
                .map_err(|err| err.to_string()),

            Message::StopProject { project_name } => self
                .requester
                .stop_project(project_name)
                .map(|_| Task::done(Message::Refresh { repeated: false }))
                .map_err(|err| err.to_string()),

            Message::StartService {
                project_name,
                service_name,
            } => self
                .requester
                .start_service(project_name, service_name)
                .map(|_| Task::done(Message::Refresh { repeated: false }))
                .map_err(|err| err.to_string()),

            Message::RestartService {
                project_name,
                service_name,
            } => self
                .requester
                .restart_service(project_name, service_name)
                .map(|_| Task::done(Message::Refresh { repeated: false }))
                .map_err(|err| err.to_string()),

            Message::StopService {
                project_name,
                service_name,
            } => self
                .requester
                .stop_service(project_name, service_name)
                .map(|_| Task::done(Message::Refresh { repeated: false }))
                .map_err(|err| {
                    format!(
                        "Unable to restart the service '{}/{}': {}",
                        project_name, service_name, err
                    )
                }),
        }
    }

    fn view(&self) -> Element<Message> {
        let view = self.page_view.view();

        let menu = Menu::new(self.project_names.clone(), self.page_view.page());
        let body = row![menu.render(), view];

        let info_bar = StatusInfoBar::new(self.requester.client().socket_path.clone());
        let status: Result<String, String> = match &self.refresh_loop_error {
            Some(err) => Err(err.clone()),
            None => self.last_action_result.clone(),
        };

        let error_bar = StatusErrorBar::new(self.last_action_at, status);

        column![error_bar.render(), body, info_bar.render()].into()
    }
}
