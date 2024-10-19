#[derive(Debug, Clone)]
pub enum Message {
    StartProject { name: String },
    StopProject { name: String },
    Refresh,
    RefreshLoop,
}
