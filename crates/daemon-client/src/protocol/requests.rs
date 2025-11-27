use crate::{
    protocol::ARG_SEPARATOR_STR, NameListResponse, NoContentResponse, ProjectInfoResponse,
    ProjectSettingsResponse, ProjectsInfoResponse, ProjectsSettingsResponse, Response,
    ServiceInfoResponse,
};

pub(crate) trait Request<R: Response> {
    fn serialize(&self) -> String;
}

pub(crate) struct ProjectsNamesRequest;

impl Request<NameListResponse> for ProjectsNamesRequest {
    fn serialize(&self) -> String {
        String::from("PROJECTS-NAMES")
    }
}

pub(crate) struct ProjectsSettingsRequest;

impl Request<ProjectsSettingsResponse> for ProjectsSettingsRequest {
    fn serialize(&self) -> String {
        String::from("PROJECTS-SETTINGS")
    }
}

pub(crate) struct ProjectsInfoRequest;

impl Request<ProjectsInfoResponse> for ProjectsInfoRequest {
    fn serialize(&self) -> String {
        String::from("PROJECTS-INFO")
    }
}

pub(crate) struct ProjectSettingsRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl Request<ProjectSettingsResponse> for ProjectSettingsRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["PROJECT-SETTINGS", self.project_name])
    }
}

pub(crate) struct ProjectInfoRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl Request<ProjectInfoResponse> for ProjectInfoRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["PROJECT-INFO", self.project_name])
    }
}

pub(crate) struct ProjectUpsertRequest<'a> {
    pub(crate) settings_json: &'a str,
}

impl Request<ProjectInfoResponse> for ProjectUpsertRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["PROJECT-UPSERT", self.settings_json])
    }
}

pub(crate) struct ProjectStartRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) env: String,
}

impl Request<ProjectInfoResponse> for ProjectStartRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["PROJECT-START", self.project_name, &self.env])
    }
}

pub(crate) struct ProjectRestartRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) env: String,
}

impl Request<ProjectInfoResponse> for ProjectRestartRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["PROJECT-RESTART", self.project_name, &self.env])
    }
}

pub(crate) struct ProjectStopRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl Request<ProjectInfoResponse> for ProjectStopRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["PROJECT-STOP", self.project_name])
    }
}

pub(crate) struct ProjectRemoveRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl Request<NoContentResponse> for ProjectRemoveRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["PROJECT-REMOVE", self.project_name])
    }
}

pub(crate) struct ProjectClearLogsRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl Request<NoContentResponse> for ProjectClearLogsRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["PROJECT-CLEAR-LOGS", self.project_name])
    }
}

pub(crate) struct ServicesNamesRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl Request<NameListResponse> for ServicesNamesRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["SERVICES-NAMES", self.project_name])
    }
}

pub(crate) struct ServiceInfoRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl Request<ServiceInfoResponse> for ServiceInfoRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["SERVICE-INFO", self.project_name, self.service_name])
    }
}

pub(crate) struct ServiceStartRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
    pub(crate) env: String,
}

impl Request<ServiceInfoResponse> for ServiceStartRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec![
            "SERVICE-START",
            self.project_name,
            self.service_name,
            &self.env,
        ])
    }
}

pub(crate) struct ServiceRestartRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
    pub(crate) env: String,
}

impl Request<ServiceInfoResponse> for ServiceRestartRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec![
            "SERVICE-RESTART",
            self.project_name,
            self.service_name,
            &self.env,
        ])
    }
}

pub(crate) struct ServiceStopRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl Request<ServiceInfoResponse> for ServiceStopRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["SERVICE-STOP", self.project_name, self.service_name])
    }
}

pub(crate) struct ServiceClearLogsRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl Request<NoContentResponse> for ServiceClearLogsRequest<'_> {
    fn serialize(&self) -> String {
        serialize_parts(vec!["SERVICE-CLEAR-LOGS", self.project_name, self.service_name])
    }
}

#[inline]
fn serialize_parts(parts: Vec<&str>) -> String {
    parts.join(ARG_SEPARATOR_STR)
}
