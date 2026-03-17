use app_config::LogPreviewMode;

#[derive(Debug, Clone)]
pub struct TuiSettings {
    pub log_preview: LogPreviewMode,
}

impl TuiSettings {
    pub fn to_info(&self) -> String {
        match self.log_preview {
            LogPreviewMode::On => String::from("Log preview: ON"),
            LogPreviewMode::Off => String::from("Log preview: OFF"),
            LogPreviewMode::Fit => String::from("Log preview: FIT"),
        }
    }
}
