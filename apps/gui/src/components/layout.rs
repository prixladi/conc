use iced::{widget::container, Background, Border, Color, Element, Length, Shadow, Theme};

use crate::message::Message;

pub struct Section<'a> {
    content: Element<'a, Message>,
    height: Length,
}

impl<'a> Section<'a> {
    pub fn new(content: Element<'a, Message>) -> Self {
        Self {
            content,
            height: Length::Fill,
        }
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<'a> From<Section<'a>> for Element<'a, Message> {
    fn from(value: Section<'a>) -> Self {
        container(
            container(value.content)
                .padding(16)
                .style(inner_container_style)
                .height(value.height),
        )
        .padding([8, 8])
        .into()
    }
}

fn inner_container_style(theme: &Theme) -> container::Style {
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
