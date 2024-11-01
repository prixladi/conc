use crate::{ProjectInfo, ServiceStatus};

impl ProjectInfo {
    pub fn running_service_count(&self) -> usize {
        self.services
            .iter()
            .filter(|service| service.status == ServiceStatus::RUNNING)
            .count()
    }

    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    pub fn any_service_running(&self) -> bool {
        self.services
            .iter()
            .any(|service| service.status == ServiceStatus::RUNNING)
    }

    pub fn all_services_running(&self) -> bool {
        self.services
            .iter()
            .all(|service| service.status == ServiceStatus::RUNNING)
    }
}
