use iced::{
    widget::{button, button::Status, column, container, row, scrollable, text, Button},
    Alignment, Background, Border, Color, Element, Length, Padding, Shadow, Theme,
};
use iced_fonts::{bootstrap::icon_to_string, Bootstrap, BOOTSTRAP_FONT};

use crate::{message::Message, pages::Page};

use super::Section;

pub struct Menu {
    projects: Vec<String>,
    current_page: Page,
}

impl Menu {
    pub fn new(projects: Vec<String>, current_page: Page) -> Self {
        Self {
            projects,
            current_page,
        }
    }
}

impl From<Menu> for Element<'_, Message> {
    fn from(value: Menu) -> Self {
        let mut project_panel = column![].spacing(8);
        for project in value.projects.iter() {
            let is_active = match &value.current_page {
                Page::Project(project_name) | Page::Service(project_name, _) => {
                    project_name == project
                }
                _ => false,
            };

            let project_name = row![text("#").size(16), text(project.clone()).size(16)].spacing(2);

            project_panel = project_panel.push(menu_button(
                project_name.into(),
                is_active,
                Message::GotoPage(Page::Project(project.clone())),
            ));
        }

        let project_button = menu_button_with_icon(
            "Projects",
            Bootstrap::HousesFill,
            value.current_page == Page::Projects,
            Message::GotoPage(Page::Projects),
        );

        let settings_button = menu_button_with_icon(
            "Settings",
            Bootstrap::Gear,
            value.current_page == Page::Settings,
            Message::GotoPage(Page::Settings),
        );

        let github_button = menu_button_with_icon(
            "Github",
            Bootstrap::Github,
            false,
            Message::OpenUrl(String::from("https://github.com/prixladi/conc")),
        );

        let panel = column![
            project_button,
            container(scrollable(project_panel)).height(Length::Fill),
            settings_button,
            github_button,
        ]
        .spacing(12);
        let content = container(panel).style(container_style).width(250);
        Section::new(content.into()).into()
    }
}

fn menu_button_with_icon(
    title: &str,
    icon: Bootstrap,
    active: bool,
    message: Message,
) -> Element<Message> {
    let icon = container(text(icon_to_string(icon)).font(BOOTSTRAP_FONT).size(18))
        .padding(Padding::default().bottom(4));

    let title_with_icon = row![icon, text(title).size(18)]
        .align_y(Alignment::Center)
        .spacing(6);

    container(menu_button(title_with_icon.into(), active, message)).into()
}

fn menu_button(title: Element<Message>, active: bool, message: Message) -> Button<Message> {
    let butt = button(title)
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

fn menu_button_active_style(theme: &Theme, _: Status) -> button::Style {
    let palette = theme.extended_palette();
    let bg_color = palette.primary.weak.color.scale_alpha(0.25);

    button::Style {
        background: Some(Background::Color(bg_color)),
        text_color: palette.background.base.text,
        border: Border::default().rounded(10),
        shadow: Shadow::default(),
    }
}
