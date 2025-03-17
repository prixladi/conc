#[derive(Debug, PartialEq, Clone)]
pub enum LogPreviewSettings {
    On,
    Off,
    Fit,
}

#[derive(Debug, Clone)]
pub struct TuiSettings {
    pub log_preview: LogPreviewSettings,
}

impl TuiSettings {
    pub fn to_info(&self) -> String {
        match self.log_preview {
            LogPreviewSettings::On => String::from("Log preview: ON"),
            LogPreviewSettings::Off => String::from("Log preview: OFF"),
            LogPreviewSettings::Fit => String::from("Log preview: FIT"),
        }
    }
}
