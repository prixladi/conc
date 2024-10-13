use std::convert::TryFrom;

use crate::{
    protocol::{
        requests::{
            ProjectInfoRequest, ProjectRemoveRequest, ProjectSettingsRequest, ProjectStartRequest,
            ProjectStopRequest, ProjectUpsertRequest, ProjectsInfoRequest, ProjectsNamesRequest,
            ProjectsSettingsRequest, Request, ServiceInfoRequest, ServiceStartRequest,
            ServiceStopRequest, ServicesNamesRequest,
        },
        responses::{
            ErrorResponse, NameListResponse, NoContentResponse, ProjectInfoResponse,
            ProjectSettingsResponse, ProjectsInfoResponse, ProjectsSettingsResponse,
            ServiceInfoResponse,
        },
    },
    socket_client::SocketClient,
};

type Response<T> = Result<T, ErrorResponse>;

pub struct Requester<'a> {
    socket_client: &'a SocketClient,
}

impl<'a> Requester<'a> {
    pub fn new(socket_client: &'a SocketClient) -> Self {
        Self { socket_client }
    }

    pub fn get_projects_names(&self) -> Response<NameListResponse> {
        self.send_request(ProjectsNamesRequest)
    }

    pub fn get_projects_settings(&self) -> Response<ProjectsSettingsResponse> {
        self.send_request(ProjectsSettingsRequest)
    }

    pub fn get_projects_info(&self) -> Response<ProjectsInfoResponse> {
        self.send_request(ProjectsInfoRequest)
    }

    pub fn upsert_project(&self, settings_json: &str) -> Response<ProjectInfoResponse> {
        self.send_request(ProjectUpsertRequest { settings_json })
    }

    pub fn get_project_settings(&self, project_name: &str) -> Response<ProjectSettingsResponse> {
        self.send_request(ProjectSettingsRequest { project_name })
    }

    pub fn get_project_info(&self, project_name: &str) -> Response<ProjectInfoResponse> {
        self.send_request(ProjectInfoRequest { project_name })
    }

    pub fn start_project(&self, project_name: &str) -> Response<ProjectInfoResponse> {
        self.send_request(ProjectStartRequest { project_name })
    }

    pub fn stop_project(&self, project_name: &str) -> Response<ProjectInfoResponse> {
        self.send_request(ProjectStopRequest { project_name })
    }

    pub fn remove_project(&self, project_name: &str) -> Response<NoContentResponse> {
        self.send_request(ProjectRemoveRequest { project_name })
    }

    pub fn get_services_names(&self, project_name: &str) -> Response<NameListResponse> {
        self.send_request(ServicesNamesRequest { project_name })
    }

    pub fn get_services_info(
        &self,
        project_name: &str,
        service_name: &str,
    ) -> Response<ServiceInfoResponse> {
        self.send_request(ServiceInfoRequest {
            project_name,
            service_name,
        })
    }

    pub fn start_service(
        &self,
        project_name: &str,
        service_name: &str,
    ) -> Response<ServiceInfoResponse> {
        self.send_request(ServiceStartRequest {
            project_name,
            service_name,
        })
    }

    pub fn stop_service(
        &self,
        project_name: &str,
        service_name: &str,
    ) -> Response<ServiceInfoResponse> {
        self.send_request(ServiceStopRequest {
            project_name,
            service_name,
        })
    }

    fn send_request<R: TryFrom<Vec<String>>>(&self, req: impl Request) -> Response<R> {
        let req_string = req.serialize();
        let resp = self.socket_client.send(req_string.as_bytes())?;
        let parts: Vec<String> = resp.split("\n").map(String::from).collect();
        R::try_from(parts.clone()).map_err(|_| ErrorResponse::from(parts))
    }
}
