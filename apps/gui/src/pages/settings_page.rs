use app_config::AppConfig;
use iced::widget::{column, pick_list, scrollable, text};
use iced::{Element, Length, Theme};

use crate::components::Section;
use crate::message::Message;
use crate::utils::prettify_serializable;

use super::{Page, PageData, PageView};

pub struct SettingsPage {
    theme: Theme,
    config: AppConfig,
}

impl SettingsPage {
    pub fn new(data: PageData) -> Self {
        Self {
            theme: data.theme,
            config: data.config,
        }
    }
}

impl PageView for SettingsPage {
    fn page(&self) -> Page {
        Page::Settings
    }

    fn refresh(&mut self, data: PageData) -> Result<(), String> {
        self.theme = data.theme;
        self.config = data.config;
        Ok(())
    }

    fn view(&self) -> Element<Message> {
        let mut view = column![];

        let theme_picker = column![
            text("Theme"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged).width(Length::Fill),
        ]
        .spacing(10);

        view = view.push(Section::new(theme_picker.into()));

        let pretty_config = prettify_serializable(&self.config).unwrap_or_default();
        let json_view = scrollable(text(pretty_config).width(Length::Fill));
        let config_section = Section::new(json_view.into());
        view = view.push(config_section);

        view.into()
    }
}
