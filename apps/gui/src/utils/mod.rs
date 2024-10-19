use daemon_client::ServiceStatus;

pub fn prettify_json<'a, T: serde::de::Deserialize<'a> + serde::Serialize>(
    data: &'a str,
) -> Result<String, serde_json::Error> {
    serde_json::from_str::<T>(data).and_then(|d| serde_json::to_string_pretty(&d))
}

pub fn service_status_stringify(status: &ServiceStatus) -> String {
    match status {
        ServiceStatus::IDLE => String::from("Idle"),
        ServiceStatus::RUNNING => String::from("Running"),
        ServiceStatus::STOPPED => String::from("Stopped"),
    }
}
