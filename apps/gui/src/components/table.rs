use iced::{
    alignment,
    widget::{column, container, horizontal_space, row, scrollable, text},
    Element,
};

use crate::message::Message;

pub struct InfoTable<'a> {
    title: String,
    names: Vec<String>,
    statuses: Vec<String>,
    actions: Vec<Element<'a, Message>>,
}

impl<'a> InfoTable<'a> {
    pub fn new(
        title: String,
        names: Vec<String>,
        statuses: Vec<String>,
        actions: Vec<Element<'a, Message>>,
    ) -> Self {
        Self {
            title,
            names,
            statuses,
            actions,
        }
    }

    pub fn render(self) -> Element<'a, Message> {
        let mut names = column!["NAME"].spacing(10);
        let mut statuses = column!["STATUS"].spacing(10);
        let mut actions = column!["ACTIONS"].spacing(10);

        for name in self.names {
            names = names.push(cell(text(name).size(18)));
        }
        for status in self.statuses {
            statuses = statuses.push(cell(text(status).size(18)));
        }
        for action in self.actions {
            actions = actions.push(action);
        }

        let rows = scrollable(
            row![
                names,
                horizontal_space(),
                statuses,
                horizontal_space(),
                actions
            ]
            .spacing(8),
        );

        column![text(self.title).size(30), rows]
            .spacing(12)
            .padding(8)
            .into()
    }
}

fn cell<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .align_y(alignment::Vertical::Bottom)
        .height(30)
        .into()
}
