use iced::{
    alignment,
    widget::{container, row, text},
    Element, Padding,
};

use crate::message::Message;

pub struct PageTitle<'a> {
    title: String,
    additional_content: Option<Element<'a, Message>>,
}

impl<'a> PageTitle<'a> {
    pub fn new(title: String) -> Self {
        Self {
            title,
            additional_content: None,
        }
    }

    pub fn additional_content(mut self, additional_content: Element<'a, Message>) -> Self {
        self.additional_content = Some(additional_content);
        self
    }
}

impl<'a> From<PageTitle<'a>> for Element<'a, Message> {
    fn from(value: PageTitle<'a>) -> Self {
        let mut row = row![text(value.title).size(30)]
            .align_y(alignment::Vertical::Center)
            .spacing(12)
            .height(30);
        if let Some(content) = value.additional_content {
            row = row.push(content);
        }

        container(row)
            .align_y(alignment::Vertical::Center)
            .height(42)
            .padding(Padding::default().bottom(12))
            .into()
    }
}
