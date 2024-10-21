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
        let theme = Theme::Ferra;

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

        (app, Task::done(Message::RefreshLoop))
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
        let is_loop = message == Message::RefreshLoop;

        let res: Result<(String, bool), String> = match message {
            Message::RefreshLoop => Ok((String::new(), true)),

            Message::GotoPage(page) => {
                self.page_view = get_page(
                    page.clone(),
                    PageData {
                        requester: self.requester.clone(),
                        theme: self.theme.clone(),
                        config: self.config.clone(),
                    },
                );
                Ok((format!("Navigated to the page '{}'", page), true))
            }

            Message::OpenUrl(url) => open::that(&url)
                .map(|_| (format!("Opened the external url '{}'", url), true))
                .map_err(|err| format!("Unable to open the external url '{}': {}", url, err)),

            Message::ThemeChanged(theme) => {
                self.theme = theme;
                Ok((format!("Changed theme to '{}'", self.theme), true))
            }

            Message::StartProject { project_name } => self
                .requester
                .start_project(&project_name)
                .map(|_| (format!("Started the project '{}'", project_name), true))
                .map_err(|err| format!("Unable to start the project '{}': {}", project_name, err)),

            Message::RestartProject { project_name } => self
                .requester
                .restart_project(&project_name)
                .map(|_| (format!("Restarted the project '{}'", project_name), true))
                .map_err(|err| {
                    format!("Unable to restart the project '{}': {}", project_name, err)
                }),

            Message::StopProject { project_name } => self
                .requester
                .stop_project(&project_name)
                .map(|_| (format!("Stopped the project '{}'", project_name), true))
                .map_err(|err| format!("Unable to stop the project '{}': {}", project_name, err)),

            Message::StartService {
                project_name,
                service_name,
            } => self
                .requester
                .start_service(&project_name, &service_name)
                .map(|_| {
                    (
                        format!("Started the service '{}/{}'", project_name, service_name),
                        true,
                    )
                })
                .map_err(|err| {
                    format!(
                        "Unable to start the service '{}/{}': {}",
                        project_name, service_name, err
                    )
                }),

            Message::RestartService {
                project_name,
                service_name,
            } => self
                .requester
                .restart_service(&project_name, &service_name)
                .map(|_| {
                    (
                        format!("Restarted the service '{}/{}'", project_name, service_name),
                        true,
                    )
                })
                .map_err(|err| {
                    format!(
                        "Unable to restart the service '{}/{}': {}",
                        project_name, service_name, err
                    )
                }),

            Message::StopService {
                project_name,
                service_name,
            } => self
                .requester
                .stop_service(&project_name, &service_name)
                .map(|_| {
                    (
                        format!("Restarted the service '{}/{}'", project_name, service_name),
                        true,
                    )
                })
                .map_err(|err| {
                    format!(
                        "Unable to restart the service '{}/{}': {}",
                        project_name, service_name, err
                    )
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

            let page_data = PageData {
                requester: self.requester.clone(),
                theme: self.theme.clone(),
                config: self.config.clone(),
            };
            if let Err(err) = self.page_view.refresh(page_data) {
                self.refresh_loop_error = Some(err);
                self.last_action_at = Local::now();
            }
        };

        if is_loop {
            return Task::perform(sleep(Duration::from_secs(10)), |_| Message::RefreshLoop);
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
