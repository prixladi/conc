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

impl<'a> Request<ProjectSettingsResponse> for ProjectSettingsRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            "PROJECT-SETTINGS", ARG_SEPARATOR_STR, self.project_name
        )
    }
}

pub(crate) struct ProjectInfoRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request<ProjectInfoResponse> for ProjectInfoRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            "PROJECT-INFO", ARG_SEPARATOR_STR, self.project_name
        )
    }
}

pub(crate) struct ProjectUpsertRequest<'a> {
    pub(crate) settings_json: &'a str,
}

impl<'a> Request<ProjectInfoResponse> for ProjectUpsertRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            "PROJECT-UPSERT", ARG_SEPARATOR_STR, self.settings_json
        )
    }
}

pub(crate) struct ProjectStartRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request<ProjectInfoResponse> for ProjectStartRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            "PROJECT-START", ARG_SEPARATOR_STR, self.project_name
        )
    }
}

pub(crate) struct ProjectRestartRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request<ProjectInfoResponse> for ProjectRestartRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            "PROJECT-RESTART", ARG_SEPARATOR_STR, self.project_name
        )
    }
}

pub(crate) struct ProjectStopRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request<ProjectInfoResponse> for ProjectStopRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            "PROJECT-STOP", ARG_SEPARATOR_STR, self.project_name
        )
    }
}

pub(crate) struct ProjectRemoveRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request<NoContentResponse> for ProjectRemoveRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            "PROJECT-REMOVE", ARG_SEPARATOR_STR, self.project_name
        )
    }
}

pub(crate) struct ServicesNamesRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request<NameListResponse> for ServicesNamesRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}",
            "SERVICES-NAMES", ARG_SEPARATOR_STR, self.project_name
        )
    }
}

pub(crate) struct ServiceInfoRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl<'a> Request<ServiceInfoResponse> for ServiceInfoRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}{}{}",
            "SERVICE-INFO",
            ARG_SEPARATOR_STR,
            self.project_name,
            ARG_SEPARATOR_STR,
            self.service_name
        )
    }
}

pub(crate) struct ServiceStartRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl<'a> Request<ServiceInfoResponse> for ServiceStartRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}{}{}",
            "SERVICE-START",
            ARG_SEPARATOR_STR,
            self.project_name,
            ARG_SEPARATOR_STR,
            self.service_name
        )
    }
}

pub(crate) struct ServiceRestartRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl<'a> Request<ServiceInfoResponse> for ServiceRestartRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}{}{}",
            "SERVICE-RESTART",
            ARG_SEPARATOR_STR,
            self.project_name,
            ARG_SEPARATOR_STR,
            self.service_name
        )
    }
}

pub(crate) struct ServiceStopRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl<'a> Request<ServiceInfoResponse> for ServiceStopRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}{}{}",
            "SERVICE-STOP",
            ARG_SEPARATOR_STR,
            self.project_name,
            ARG_SEPARATOR_STR,
            self.service_name
        )
    }
}
