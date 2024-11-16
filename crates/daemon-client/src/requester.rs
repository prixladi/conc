use std::collections::HashMap;

use crate::{
    protocol::{
        requests::{
            ProjectInfoRequest, ProjectRemoveRequest, ProjectRestartRequest,
            ProjectSettingsRequest, ProjectStartRequest, ProjectStopRequest, ProjectUpsertRequest,
            ProjectsInfoRequest, ProjectsNamesRequest, ProjectsSettingsRequest, Request,
            ServiceInfoRequest, ServiceRestartRequest, ServiceStartRequest, ServiceStopRequest,
            ServicesNamesRequest,
        },
        responses::ErrorResponse,
        ARG_SEPARATOR_STR,
    },
    socket_client::SocketClient,
    ProjectInfo, Response, ServiceInfo,
};

type Res<T> = Result<T, ErrorResponse>;

#[derive(Debug, Clone)]
pub struct Requester {
    socket_client: SocketClient,
    use_caller_env: bool,
}

impl Requester {
    pub fn new(socket_client: SocketClient, use_caller_env: bool) -> Self {
        Self {
            socket_client,
            use_caller_env,
        }
    }

    pub fn client(&self) -> &SocketClient {
        &self.socket_client
    }

    pub fn get_project_names(&self) -> Res<Vec<String>> {
        self.send_request(ProjectsNamesRequest)
            .map(|res| res.values)
    }

    pub fn get_projects_settings(&self) -> Res<Vec<(String, String)>> {
        self.send_request(ProjectsSettingsRequest)
            .map(|res| res.values)
    }

    pub fn get_projects_info(&self) -> Res<Vec<ProjectInfo>> {
        self.send_request(ProjectsInfoRequest).map(|res| res.values)
    }

    pub fn upsert_project(&self, settings_json: &str) -> Res<ProjectInfo> {
        self.send_request(ProjectUpsertRequest { settings_json })
            .map(|res| res.value)
    }

    pub fn get_project_settings(&self, project_name: &str) -> Res<String> {
        self.send_request(ProjectSettingsRequest { project_name })
            .map(|res| res.value)
    }

    pub fn get_project_info(&self, project_name: &str) -> Res<ProjectInfo> {
        self.send_request(ProjectInfoRequest { project_name })
            .map(|res| res.value)
    }

    pub fn start_project(&self, project_name: &str) -> Res<ProjectInfo> {
        self.send_request(ProjectStartRequest {
            project_name,
            env: self.get_request_env(),
        })
        .map(|res| res.value)
    }

    pub fn restart_project(&self, project_name: &str) -> Res<ProjectInfo> {
        self.send_request(ProjectRestartRequest {
            project_name,
            env: self.get_request_env(),
        })
        .map(|res| res.value)
    }

    pub fn stop_project(&self, project_name: &str) -> Res<ProjectInfo> {
        self.send_request(ProjectStopRequest { project_name })
            .map(|res| res.value)
    }

    pub fn remove_project(&self, project_name: &str) -> Res<()> {
        self.send_request(ProjectRemoveRequest { project_name })
            .map(|_| ())
    }

    pub fn get_service_names(&self, project_name: &str) -> Res<Vec<String>> {
        self.send_request(ServicesNamesRequest { project_name })
            .map(|res| res.values)
    }

    pub fn get_services_info(&self, project_name: &str, service_name: &str) -> Res<ServiceInfo> {
        self.send_request(ServiceInfoRequest {
            project_name,
            service_name,
        })
        .map(|res| res.value)
    }

    pub fn start_service(&self, project_name: &str, service_name: &str) -> Res<ServiceInfo> {
        self.send_request(ServiceStartRequest {
            project_name,
            service_name,
            env: self.get_request_env(),
        })
        .map(|res| res.value)
    }

    pub fn restart_service(&self, project_name: &str, service_name: &str) -> Res<ServiceInfo> {
        self.send_request(ServiceRestartRequest {
            project_name,
            service_name,
            env: self.get_request_env(),
        })
        .map(|res| res.value)
    }

    pub fn stop_service(&self, project_name: &str, service_name: &str) -> Res<ServiceInfo> {
        self.send_request(ServiceStopRequest {
            project_name,
            service_name,
        })
        .map(|res| res.value)
    }

    fn send_request<R: Response>(&self, req: impl Request<R>) -> Res<R> {
        let req_string = req.serialize();
        let resp = self.socket_client.send(req_string.as_bytes())?;
        let parts: Vec<String> = resp.split(ARG_SEPARATOR_STR).map(String::from).collect();
        R::try_from(parts.clone()).map_err(|_| ErrorResponse::from(parts))
    }

    fn get_request_env(&self) -> String {
        if self.use_caller_env {
            let map: HashMap<String, String> = std::env::vars().collect();
            serde_json::to_string(&map).unwrap()
        } else {
            String::from("{}")
        }
    }
}
