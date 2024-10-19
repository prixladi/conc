use crate::pages::Page;

#[derive(Debug, Clone)]
pub enum Message {
    StartProject { name: String },
    RestartProject { name: String },
    StopProject { name: String },
    GotoPage { page: Page },
    RefreshLoop,
}
