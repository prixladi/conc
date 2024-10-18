use chrono::{DateTime, Local};
use iced::{
    widget::{container, row, text},
    Background, Border, Element, Length, Shadow, Theme,
};

pub struct StatusErrorBar {
    error: Option<String>,
}

impl StatusErrorBar {
    pub fn new(error: Option<String>) -> Self {
        Self { error }
    }

    pub fn render<'a, Message: 'a>(self) -> Element<'a, Message> {
        let is_error = self.error.is_some();
        let mut status_bar = row![].height(32).width(Length::Fill).spacing(20).padding(8);

        if let Some(err) = &self.error {
            status_bar = status_bar.push(text(err.clone()).size(16));
        }

        let status_bar_container = if is_error {
            container(status_bar).style(error_container_style)
        } else {
            container(status_bar).style(info_container_style)
        };

        status_bar_container.into()
    }
}

pub struct StatusInfoBar {
    last_refresh_at: DateTime<Local>,
    socket_path: String,
}

impl StatusInfoBar {
    pub fn new(last_refresh_at: DateTime<Local>, socket_path: String) -> Self {
        Self {
            last_refresh_at,
            socket_path,
        }
    }

    pub fn render<'a, Message: 'a>(self) -> Element<'a, Message> {
        let formatted_date =
            text(self.last_refresh_at.format("%d/%m/%Y %H:%M:%S").to_string()).size(16);
        let socket = format!("Connected to the daemon socket at unix://{}", self.socket_path);
        let status_bar = row![formatted_date, text("|").size(16), text(socket).size(16)]
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
