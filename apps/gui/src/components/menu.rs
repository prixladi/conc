use iced::{
    widget::{button, button::Status, column, container, scrollable, text, Button},
    Background, Border, Color, Element, Length, Padding, Shadow, Theme,
};

use crate::message::Message;

pub struct Menu {
    projects: Vec<(String, bool)>,
}

impl<'a> Menu {
    pub fn new(projects: Vec<(String, bool)>) -> Self {
        Self { projects }
    }

    pub fn render(self) -> Element<'a, Message> {
        let mut panel = column![].spacing(2);
        panel = panel.push(self.button(String::from("Projects"), 18, true, Message::Refresh));

        for (project, active) in self.projects.iter() {
            panel = panel.push(self.button(format!("#{}", project), 16, *active, Message::Refresh));
        }

        scrollable(
            container(panel)
                .style(container_style)
                .padding(16)
                .width(200),
        )
        .into()
    }

    fn button(
        &self,
        title: String,
        txt_size: i32,
        active: bool,
        message: Message,
    ) -> Button<'a, Message> {
        let txt = text(title).size(txt_size as f32).center();
        let butt = button(txt)
            .width(Length::Fill)
            .padding(Padding::default().top(8).bottom(4).left(8).right(8))
            .on_press(message);

        match active {
            true => butt.style(menu_button_active_style),
            false => butt.style(menu_button_style),
        }
    }
}

fn container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text),
        background: Some(Background::Color(palette.background.base.color)),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

fn menu_button_style(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: palette.background.base.text,
        border: Border::default().rounded(10),
        shadow: Shadow::default(),
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Pressed => button::Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            text_color: palette.background.strong.text,
            ..base
        },
        Status::Disabled => base,
    }
}

fn menu_button_active_style(theme: &Theme, _: Status) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        background: Some(Background::Color(palette.primary.strong.color)),
        text_color: palette.background.strong.text,
        border: Border::default().rounded(10),
        shadow: Shadow::default(),
    }
}
