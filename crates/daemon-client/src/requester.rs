use crate::{
    protocol::{
        requests::{
            ProjectInfoRequest, ProjectRemoveRequest, ProjectRestartRequest,
            ProjectSettingsRequest, ProjectStartRequest, ProjectStopRequest, ProjectUpsertRequest,
            ProjectsInfoRequest, ProjectsNamesRequest, ProjectsSettingsRequest, Request,
            ServiceInfoRequest, ServiceRestartRequest, ServiceStartRequest, ServiceStopRequest,
            ServicesNamesRequest,
        },
        responses::{
            ErrorResponse, NameListResponse, NoContentResponse, ProjectInfoResponse,
            ProjectSettingsResponse, ProjectsInfoResponse, ProjectsSettingsResponse,
            ServiceInfoResponse,
        },
        ARG_SEPARATOR_STR,
    },
    socket_client::SocketClient,
    Response,
};

type Res<T> = Result<T, ErrorResponse>;

#[derive(Clone)]
pub struct Requester {
    socket_client: SocketClient,
}

impl Requester {
    pub fn new(socket_client: SocketClient) -> Self {
        Self { socket_client }
    }

    pub fn client(&self) -> &SocketClient {
        &self.socket_client
    }

    pub fn get_project_names(&self) -> Res<NameListResponse> {
        self.send_request(ProjectsNamesRequest)
    }

    pub fn get_projects_settings(&self) -> Res<ProjectsSettingsResponse> {
        self.send_request(ProjectsSettingsRequest)
    }

    pub fn get_projects_info(&self) -> Res<ProjectsInfoResponse> {
        self.send_request(ProjectsInfoRequest)
    }

    pub fn upsert_project(&self, settings_json: &str) -> Res<ProjectInfoResponse> {
        self.send_request(ProjectUpsertRequest { settings_json })
    }

    pub fn get_project_settings(&self, project_name: &str) -> Res<ProjectSettingsResponse> {
        self.send_request(ProjectSettingsRequest { project_name })
    }

    pub fn get_project_info(&self, project_name: &str) -> Res<ProjectInfoResponse> {
        self.send_request(ProjectInfoRequest { project_name })
    }

    pub fn start_project(&self, project_name: &str) -> Res<ProjectInfoResponse> {
        self.send_request(ProjectStartRequest { project_name })
    }

    pub fn restart_project(&self, project_name: &str) -> Res<ProjectInfoResponse> {
        self.send_request(ProjectRestartRequest { project_name })
    }

    pub fn stop_project(&self, project_name: &str) -> Res<ProjectInfoResponse> {
        self.send_request(ProjectStopRequest { project_name })
    }

    pub fn remove_project(&self, project_name: &str) -> Res<NoContentResponse> {
        self.send_request(ProjectRemoveRequest { project_name })
    }

    pub fn get_service_names(&self, project_name: &str) -> Res<NameListResponse> {
        self.send_request(ServicesNamesRequest { project_name })
    }

    pub fn get_services_info(
        &self,
        project_name: &str,
        service_name: &str,
    ) -> Res<ServiceInfoResponse> {
        self.send_request(ServiceInfoRequest {
            project_name,
            service_name,
        })
    }

    pub fn start_service(
        &self,
        project_name: &str,
        service_name: &str,
    ) -> Res<ServiceInfoResponse> {
        self.send_request(ServiceStartRequest {
            project_name,
            service_name,
        })
    }

    pub fn restart_service(
        &self,
        project_name: &str,
        service_name: &str,
    ) -> Res<ServiceInfoResponse> {
        self.send_request(ServiceRestartRequest {
            project_name,
            service_name,
        })
    }

    pub fn stop_service(&self, project_name: &str, service_name: &str) -> Res<ServiceInfoResponse> {
        self.send_request(ServiceStopRequest {
            project_name,
            service_name,
        })
    }

    fn send_request<R: Response>(&self, req: impl Request<R>) -> Res<R> {
        let req_string = req.serialize();
        let resp = self.socket_client.send(req_string.as_bytes())?;
        let parts: Vec<String> = resp.split(ARG_SEPARATOR_STR).map(String::from).collect();
        R::try_from(parts.clone()).map_err(|_| ErrorResponse::from(parts))
    }
}
