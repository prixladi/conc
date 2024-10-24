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

        let page_view = get_page(Page::Projects);

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

    fn create_page_data(&self) -> PageData {
        PageData {
            requester: self.requester.clone(),
            theme: self.theme.clone(),
            config: self.config.clone(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let is_refresh_loop = message == Message::RefreshLoop;

        let res = handle_message(self, &message);
        if let Err(err) = res {
            self.last_action_result = Err(message.to_error_message(&err));
            self.last_action_at = Local::now();
            return Task::none();
        }

        if !is_refresh_loop {
            self.last_action_result = Ok(message.to_success_message());
            self.last_action_at = Local::now();
        };

        match res.unwrap() {
            UpdateAction::None => Task::none(),
            UpdateAction::Task(task) => task,
            UpdateAction::Refresh => {
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

                if let Err(err) = self.page_view.refresh(self.create_page_data()) {
                    self.refresh_loop_error = Some(err);
                    self.last_action_at = Local::now();
                };

                match is_refresh_loop {
                    true => Task::perform(sleep(Duration::from_secs(1)), |_| Message::RefreshLoop),
                    false => Task::none(),
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let view = self.page_view.view();

        let menu = Menu::new(self.project_names.clone(), self.page_view.page());
        let body = row![menu, view];

        let info_bar = StatusInfoBar::new(self.requester.client().socket_path.clone());
        let status: Result<String, String> = match &self.refresh_loop_error {
            Some(err) => Err(err.clone()),
            None => self.last_action_result.clone(),
        };

        let error_bar = StatusErrorBar::new(self.last_action_at, status);

        column![error_bar, body, info_bar].into()
    }
}

enum UpdateAction {
    Task(Task<Message>),
    Refresh,
    None,
}

impl From<Task<Message>> for UpdateAction {
    fn from(value: Task<Message>) -> Self {
        UpdateAction::Task(value)
    }
}

fn handle_message(app: &mut App, message: &Message) -> Result<UpdateAction, String> {
    match message {
        Message::RefreshLoop => Ok(UpdateAction::Refresh),

        Message::GotoPage(page) => {
            app.page_view = get_page(page.clone());
            Ok(UpdateAction::Refresh)
        }

        Message::OpenUrl(url) => open::that(url)
            .map(|_| UpdateAction::None)
            .map_err(|err| err.to_string()),

        Message::ThemeChanged(theme) => {
            app.theme = theme.clone();
            Ok(UpdateAction::Refresh)
        }

        Message::CopyToClipboard { name: _, data } => {
            Ok(iced::clipboard::write(data.clone()).into())
        }

        Message::StartProject { project_name } => app
            .requester
            .start_project(project_name)
            .map(|_| UpdateAction::Refresh)
            .map_err(|err| err.to_string()),

        Message::RestartProject { project_name } => app
            .requester
            .restart_project(project_name)
            .map(|_| UpdateAction::Refresh)
            .map_err(|err| err.to_string()),

        Message::StopProject { project_name } => app
            .requester
            .stop_project(project_name)
            .map(|_| UpdateAction::Refresh)
            .map_err(|err| err.to_string()),

        Message::StartService {
            project_name,
            service_name,
        } => app
            .requester
            .start_service(project_name, service_name)
            .map(|_| UpdateAction::Refresh)
            .map_err(|err| err.to_string()),

        Message::RestartService {
            project_name,
            service_name,
        } => app
            .requester
            .restart_service(project_name, service_name)
            .map(|_| UpdateAction::Refresh)
            .map_err(|err| err.to_string()),

        Message::StopService {
            project_name,
            service_name,
        } => app
            .requester
            .stop_service(project_name, service_name)
            .map(|_| UpdateAction::Refresh)
            .map_err(|err| {
                format!(
                    "Unable to restart the service '{}/{}': {}",
                    project_name, service_name, err
                )
            }),
    }
}
