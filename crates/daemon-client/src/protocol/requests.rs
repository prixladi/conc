pub(crate) trait Request {
    fn serialize(&self) -> String;
}

pub(crate) struct ProjectsNamesRequest;

impl Request for ProjectsNamesRequest {
    fn serialize(&self) -> String {
        String::from("PROJECTS-NAMES")
    }
}

pub(crate) struct ProjectsSettingsRequest;

impl Request for ProjectsSettingsRequest {
    fn serialize(&self) -> String {
        String::from("PROJECTS-SETTINGS")
    }
}

pub(crate) struct ProjectsInfoRequest;

impl Request for ProjectsInfoRequest {
    fn serialize(&self) -> String {
        String::from("PROJECTS-INFO")
    }
}

pub(crate) struct ProjectSettingsRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request for ProjectSettingsRequest<'a> {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-SETTINGS", self.project_name)
    }
}

pub(crate) struct ProjectInfoRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request for ProjectInfoRequest<'a> {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-INFO", self.project_name)
    }
}

pub(crate) struct ProjectUpsertRequest<'a> {
    pub(crate) settings_json: &'a str,
}

impl<'a> Request for ProjectUpsertRequest<'a> {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-UPSERT", self.settings_json)
    }
}

pub(crate) struct ProjectStartRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request for ProjectStartRequest<'a> {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-START", self.project_name)
    }
}

pub(crate) struct ProjectStopRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request for ProjectStopRequest<'a> {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-STOP", self.project_name)
    }
}

pub(crate) struct ProjectRemoveRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request for ProjectRemoveRequest<'a> {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-REMOVE", self.project_name)
    }
}

pub(crate) struct ServicesNamesRequest<'a> {
    pub(crate) project_name: &'a str,
}

impl<'a> Request for ServicesNamesRequest<'a> {
    fn serialize(&self) -> String {
        format!("{}\n{}", "SERVICES-NAMES", self.project_name)
    }
}

pub(crate) struct ServiceInfoRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl<'a> Request for ServiceInfoRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}\n{}\n{}",
            "SERVICE-INFO", self.project_name, self.service_name
        )
    }
}

pub(crate) struct ServiceStartRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl<'a> Request for ServiceStartRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}\n{}\n{}",
            "SERVICE-START", self.project_name, self.service_name
        )
    }
}

pub(crate) struct ServiceStopRequest<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) service_name: &'a str,
}

impl<'a> Request for ServiceStopRequest<'a> {
    fn serialize(&self) -> String {
        format!(
            "{}\n{}\n{}",
            "SERVICE-STOP", self.project_name, self.service_name
        )
    }
}
