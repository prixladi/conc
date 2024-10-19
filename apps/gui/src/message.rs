use crate::pages::Page;

#[derive(Debug, Clone)]
pub enum Message {
    StartProject { name: String },
    RestartProject { name: String },
    StopProject { name: String },
    StartService { project: String, name: String },
    RestartService { project: String, name: String },
    StopService { project: String, name: String },
    GotoPage { page: Page },
    RefreshLoop,
}
