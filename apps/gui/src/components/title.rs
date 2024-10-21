use iced::{
    alignment,
    widget::{container, row, text},
    Element, Padding,
};

use crate::message::Message;

pub struct PageTitle<'a> {
    title: String,
    content: Option<Element<'a, Message>>,
}

impl<'a> PageTitle<'a> {
    pub fn new(title: String, content: Option<Element<'a, Message>>) -> Self {
        Self { title, content }
    }

    pub fn render(self) -> Element<'a, Message> {
        let mut row = row![text(self.title).size(30)]
            .align_y(alignment::Vertical::Center)
            .spacing(12)
            .height(30);
        if let Some(content) = self.content {
            row = row.push(content);
        }

        container(row)
            .align_y(alignment::Vertical::Center)
            .height(42)
            .padding(Padding::default().bottom(12))
            .into()
    }
}
