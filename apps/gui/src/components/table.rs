use iced::{
    alignment,
    widget::{button, button::Status, column, container, horizontal_space, row, scrollable, text},
    Background, Border, Color, Element, Padding, Shadow, Theme,
};

use crate::message::Message;

pub struct InfoTable<'a, F: Fn(&str) -> Message> {
    title: Element<'a, Message>,
    names: Vec<String>,
    statuses: Vec<String>,
    actions: Vec<Element<'a, Message>>,
    name_to_message: F,
}

impl<'a, F: Fn(&str) -> Message> InfoTable<'a, F> {
    pub fn new(
        title: Element<'a, Message>,
        names: Vec<String>,
        statuses: Vec<String>,
        actions: Vec<Element<'a, Message>>,
        name_to_message: F,
    ) -> Self {
        Self {
            title,
            names,
            statuses,
            actions,
            name_to_message,
        }
    }
}

impl<'a, F: Fn(&str) -> Message> From<InfoTable<'a, F>> for Element<'a, Message> {
    fn from(value: InfoTable<'a, F>) -> Self {
        let mut names = column![column_tile("NAME", 8)].spacing(10);
        let mut statuses = column![column_tile("STATUS", 0)].spacing(10);
        let mut actions = column![column_tile("ACTIONS", 0)]
            .spacing(10)
            .padding(Padding::default().right(15));

        for name in value.names {
            names = names.push(name_button(name, &value.name_to_message));
        }
        for status in value.statuses {
            statuses = statuses.push(cell(text(status).size(18).into()));
        }
        for action in value.actions {
            actions = actions.push(cell(action));
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

        column![value.title, rows].spacing(12).into()
    }
}

fn column_tile(text: &str, padding_left: i32) -> Element<Message> {
    container(text)
        .padding(Padding::default().left(padding_left as f32))
        .into()
}

fn name_button<'a>(
    name: String,
    name_to_message: impl Fn(&str) -> Message,
) -> Element<'a, Message> {
    let message = name_to_message(&name);
    let txt = cell(text(name).size(18).into());
    button(txt)
        .style(name_button_style)
        .padding(Padding::default().top(6).bottom(4).left(8).right(8))
        .height(30)
        .on_press(message)
        .into()
}

fn cell(content: Element<'_, Message>) -> Element<'_, Message> {
    container(content)
        .align_y(alignment::Vertical::Bottom)
        .height(30)
        .into()
}

fn name_button_style(theme: &Theme, status: Status) -> button::Style {
    let palette = theme.extended_palette();
    let bg_color = palette.primary.weak.color.scale_alpha(0.25);

    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: palette.background.base.text,
        border: Border::default().rounded(10),
        shadow: Shadow::default(),
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Pressed => button::Style {
            background: Some(Background::Color(bg_color)),
            ..base
        },
        Status::Disabled => base,
    }
}
