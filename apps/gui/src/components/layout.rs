use iced::{widget::container, Background, Border, Color, Element, Length, Shadow, Theme};

use crate::message::Message;

pub struct Section<'a> {
    content: Element<'a, Message>,
}

impl<'a> Section<'a> {
    pub fn new(content: Element<'a, Message>) -> Self {
        Self { content }
    }

    pub fn render(self) -> Element<'a, Message> {
        container(
            container(self.content)
                .padding(16)
                .style(outer_container_style)
                .height(Length::Fill),
        )
        .padding([8, 8])
        .into()
    }
}

fn outer_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let scale = match palette.is_dark {
        true => 0.1,
        false => 0.5,
    };
    let border_color = palette.background.strong.color.scale_alpha(scale);

    container::Style {
        text_color: None,
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default().rounded(20).width(1).color(border_color),
        shadow: Shadow::default(),
    }
}
