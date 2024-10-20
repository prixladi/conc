use iced::{widget::container, Background, Border, Color, Element, Length, Shadow, Theme};

use crate::message::Message;

pub struct Section;

impl<'a> Section {
    pub fn new() -> Self {
        Self
    }

    pub fn render(self, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        container(
            container(content)
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

    let border_color = palette.background.strong.color.scale_alpha(0.1);

    container::Style {
        text_color: None,
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default().rounded(20).width(1).color(border_color),
        shadow: Shadow::default(),
    }
}
