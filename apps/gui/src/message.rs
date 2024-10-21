use crate::pages::Page;

#[derive(Debug, Clone)]
pub enum Message {
    StartProject {
        project_name: String,
    },
    RestartProject {
        project_name: String,
    },
    StopProject {
        project_name: String,
    },
    StartService {
        project_name: String,
        service_name: String,
    },
    RestartService {
        project_name: String,
        service_name: String,
    },
    StopService {
        project_name: String,
        service_name: String,
    },
    GotoPage(Page),
    OpenUrl(String),
    RefreshLoop,
}
