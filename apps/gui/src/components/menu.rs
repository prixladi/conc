use iced::{
    widget::{button, button::Status, column, container, scrollable, text, Button},
    Background, Border, Color, Element, Length, Padding, Shadow, Theme,
};

use crate::{message::Message, pages::Page};

use super::Section;

pub struct Menu {
    projects: Vec<String>,
    current_page: Page,
}

impl<'a> Menu {
    pub fn new(projects: Vec<String>, current_page: Page) -> Self {
        Self {
            projects,
            current_page,
        }
    }

    pub fn render(self) -> Element<'a, Message> {
        let mut panel = column![].spacing(8);
        panel = panel.push(menu_button(
            String::from("Projects"),
            18,
            Page::Projects == self.current_page,
            Message::GotoPage {
                page: Page::Projects,
            },
        ));

        for project in self.projects.iter() {
            let is_active = match &self.current_page {
                Page::Project(project_name) | Page::Service(project_name, _) => {
                    project_name == project
                }
                _ => false,
            };

            panel = panel.push(menu_button(
                format!("#{}", project),
                16,
                is_active,
                Message::GotoPage {
                    page: Page::Project(project.clone()),
                },
            ));
        }

        let content = scrollable(
            container(panel)
                .style(container_style)
                .width(250),
        );
        Section::new().render(content)
    }
}

fn menu_button<'a>(
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
    let bg_color = palette.primary.weak.color.scale_alpha(0.05);

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

fn menu_button_active_style(theme: &Theme, _: Status) -> button::Style {
    let palette = theme.extended_palette();
    let bg_color = palette.primary.weak.color.scale_alpha(0.05);

    button::Style {
        background: Some(Background::Color(bg_color)),
        text_color: palette.background.base.text,
        border: Border::default().rounded(10),
        shadow: Shadow::default(),
    }
}
