use chrono::{DateTime, Local};
use iced::{
    widget::{container, row, text},
    Background, Border, Element, Length, Shadow, Theme,
};

pub struct StatusErrorBar {
    last_action_at: DateTime<Local>,
    status: Result<String, String>,
}

impl StatusErrorBar {
    pub fn new(last_action_at: DateTime<Local>, status: Result<String, String>) -> Self {
        Self {
            last_action_at,
            status,
        }
    }

    pub fn render<'a, Message: 'a>(self) -> Element<'a, Message> {
        let is_error = self.status.is_err();
        let formatted_date =
            text(self.last_action_at.format("%d/%m/%Y %H:%M:%S").to_string()).size(16);

        let mut status_bar = row![formatted_date, "-"]
            .height(32)
            .width(Length::Fill)
            .spacing(10)
            .padding(8);

        status_bar = status_bar.push(text(self.status.unwrap_or_else(|e| e)).size(16));

        let status_bar_container = match is_error {
            true => container(status_bar).style(error_container_style),
            false => container(status_bar).style(info_container_style),
        };

        status_bar_container.into()
    }
}

pub struct StatusInfoBar {
    socket_path: String,
}

impl StatusInfoBar {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub fn render<'a, Message: 'a>(self) -> Element<'a, Message> {
        let formatted_version = text(format!("v{}", env!("CARGO_PKG_VERSION")));
        let socket = format!("Using the daemon socket at unix://{}", self.socket_path);
        let status_bar = row![formatted_version, text("|").size(16), text(socket).size(16)]
            .height(32)
            .width(Length::Fill)
            .spacing(10)
            .padding(8);

        container(status_bar).style(info_container_style).into()
    }
}

fn error_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.danger.weak.text),
        background: Some(Background::Color(palette.danger.weak.color)),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

fn info_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}
