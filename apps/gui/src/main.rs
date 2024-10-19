use std::time::Duration;

use chrono::{DateTime, Local};
use components::{Menu, StatusErrorBar, StatusInfoBar};
use daemon_client::{ProjectInfo, Requester, ServiceStatus, SocketClient};
use iced::widget::{button, center, column, container, row, scrollable, text, Column};
use iced::{alignment, Element, Length, Padding, Task, Theme};
use message::Message;

mod components;
mod message;

pub fn main() -> iced::Result {
    iced::application("ConcG | Projects", App::update, App::view)
        .theme(|_| App::theme())
        .run_with(App::new)
}

struct App {
    requester: Requester,
    last_refresh_at: DateTime<Local>,
    error: Option<String>,
    projects: Vec<ProjectInfo>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let socket_client = SocketClient::new("../daemon/run/conc.sock");
        let requester = Requester::new(socket_client);

        let app = Self {
            requester,
            last_refresh_at: Local::now(),
            projects: vec![],
            error: None,
        };

        (app, Task::done(Message::RefreshLoop))
    }
}

impl App {
    fn theme() -> Theme {
        Theme::Ferra
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let refresh = match &message {
            Message::RefreshLoop | Message::Refresh => true,
            Message::StartProject { name } => {
                self.requester.start_project(name).unwrap();
                true
            }
            Message::StopProject { name } => {
                self.requester.stop_project(name).unwrap();
                true
            }
        };

        if refresh {
            match self.requester.get_projects_info() {
                Ok(info) => {
                    self.projects = info.values;
                    self.error = None;
                }
                Err(err) => self.error = Some(err.to_string()),
            }
            self.last_refresh_at = Local::now()
        };

        if let Message::RefreshLoop = message {
            return Task::perform(tokio::time::sleep(Duration::from_secs(1)), |_| {
                Message::RefreshLoop
            });
        };
        Task::none()
    }

    fn cell(content: Column<'_, Message>) -> Column<'_, Message> {
        column![container(content)
            .height(Length::Fill)
            .align_y(alignment::Vertical::Bottom),]
        .height(30)
    }

    fn view(&self) -> Element<Message> {
        let mut names = column!().spacing(10);
        let mut statuses = column!().spacing(10);
        let mut start_buttons = column!().spacing(10);
        let mut stop_buttons = column!().spacing(10);

        for project in &self.projects {
            let services_count = project.services.len();
            let running_services_count = project
                .services
                .iter()
                .filter(|service| service.status == ServiceStatus::RUNNING)
                .count();

            let name = Self::cell(column![text(&project.name).size(25)]);
            let status = Self::cell(column![text(format!(
                "{}/{} services running",
                running_services_count, services_count
            ))
            .size(20)]);

            let mut start_button = button("Start")
                .style(button::success)
                .padding(Padding::default().top(7).bottom(2).left(4).right(4));
            if services_count > running_services_count {
                start_button = start_button.on_press(Message::StartProject {
                    name: project.name.clone(),
                })
            }

            let mut stop_button = button("Stop")
                .style(button::danger)
                .padding(Padding::default().top(7).bottom(2).left(4).right(4));
            if running_services_count > 0 {
                stop_button = stop_button.on_press(Message::StopProject {
                    name: project.name.clone(),
                })
            }

            names = names.push(name);
            statuses = statuses.push(status);
            start_buttons = start_buttons.push(Self::cell(column![start_button]));
            stop_buttons = stop_buttons.push(Self::cell(column![stop_button]));
        }

        let rows = scrollable(row![names, statuses, start_buttons, stop_buttons].spacing(10));

        let content = center(
            column![text("Projects").size(50), rows]
                .spacing(20)
                .padding(20)
                .max_width(600),
        );

        let view = container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center);

        let menu = Menu::new(
            self.projects
                .iter()
                .map(|project| (project.name.clone(), false))
                .collect(),
        );
        let body = row![menu.render(), view];

        let info_bar = StatusInfoBar::new(
            self.last_refresh_at,
            self.requester.client().socket_path.clone(),
        );
        let error_bar = StatusErrorBar::new(self.error.clone());

        column![error_bar.render(), body, info_bar.render(),].into()
    }
}
