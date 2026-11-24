//! Public JSON shapes of the API. Kept separate from daemon-client types so the
//! HTTP contract stays stable if the socket protocol changes.

use daemon_client::{ProjectInfo, ServiceInfo, ServiceStatus};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Idle,
    Running,
    Stopped,
    Exited,
}

#[derive(Debug, Serialize)]
pub struct Service {
    pub name: String,
    pub status: Status,
    /// Present only while running.
    pub pid: Option<i32>,
    /// Unix timestamp in seconds.
    pub started_at: Option<u64>,
    /// Unix timestamp in seconds.
    pub stopped_at: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct Project {
    pub name: String,
    pub running: usize,
    pub total: usize,
    pub services: Vec<Service>,
}

impl From<ServiceInfo> for Service {
    fn from(s: ServiceInfo) -> Self {
        let status = match s.status {
            ServiceStatus::IDLE => Status::Idle,
            ServiceStatus::RUNNING => Status::Running,
            ServiceStatus::STOPPED => Status::Stopped,
            ServiceStatus::EXITED => Status::Exited,
        };
        let running = s.status == ServiceStatus::RUNNING;
        Self {
            name: s.name,
            status,
            pid: (running && s.pid > 0).then_some(s.pid),
            started_at: (s.start_time > 0).then_some(s.start_time),
            stopped_at: (s.stop_time > 0).then_some(s.stop_time),
        }
    }
}

impl From<ProjectInfo> for Project {
    fn from(p: ProjectInfo) -> Self {
        Self {
            running: p.running_service_count(),
            total: p.service_count(),
            name: p.name,
            services: p.services.into_iter().map(Service::from).collect(),
        }
    }
}
