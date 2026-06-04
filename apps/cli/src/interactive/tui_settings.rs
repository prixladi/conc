use app_config::{LogPreviewMode, RestartType};

#[derive(Debug, Clone)]
pub struct TuiSettings {
    pub log_preview: LogPreviewMode,
    pub service_restart_behavior: RestartType,
}

impl TuiSettings {
    pub fn to_short_info(&self) -> String {
        let mut info = match self.log_preview {
            LogPreviewMode::On => String::from("Log preview: on"),
            LogPreviewMode::Off => String::from("Log preview: off"),
            LogPreviewMode::Fit => String::from("Log preview: fit"),
        };

        info += match self.service_restart_behavior {
            RestartType::Hot => " | Service restart behavior: hot",
            RestartType::Cold => " | Service restart behavior: cold",
        };

        info
    }
}
