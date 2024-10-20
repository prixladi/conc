use iced::{
    alignment,
    widget::{container, row, text}, Element, Padding,
};

use crate::message::Message;

pub struct PageTitle {
    title: String,
}

impl<'a> PageTitle {
    pub fn new(title: String) -> Self {
        Self { title }
    }

    pub fn render(self, content: Option<Element<'a, Message>>) -> Element<'a, Message> {
        let mut row = row![text(self.title).size(30)]
            .align_y(alignment::Vertical::Center)
            .spacing(12)
            .height(30);
        if let Some(content) = content {
            row = row.push(content);
        }

        container(row)
            .align_y(alignment::Vertical::Center)
            .height(42)
            .padding(Padding::default().bottom(12))
            .into()
    }
}
