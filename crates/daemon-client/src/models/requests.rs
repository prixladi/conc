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

pub(crate) struct ProjectSettingsRequest {
    pub(crate) project_name: String,
}

impl Request for ProjectSettingsRequest {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-SETTINGS", self.project_name)
    }
}

pub(crate) struct ProjectInfoRequest {
    pub(crate) project_name: String,
}

impl Request for ProjectInfoRequest {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-INFO", self.project_name)
    }
}

pub(crate) struct ProjectUpsertRequest {
    pub(crate) settings_json: String,
}

impl Request for ProjectUpsertRequest {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-UPSERT", self.settings_json)
    }
}

pub(crate) struct ProjectStartRequest {
    pub(crate) project_name: String,
}

impl Request for ProjectStartRequest {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-START", self.project_name)
    }
}

pub(crate) struct ProjectStopRequest {
    pub(crate) project_name: String,
}

impl Request for ProjectStopRequest {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-STOP", self.project_name)
    }
}

pub(crate) struct ProjectRemoveRequest {
    pub(crate) project_name: String,
}

impl Request for ProjectRemoveRequest {
    fn serialize(&self) -> String {
        format!("{}\n{}", "PROJECT-REMOVE", self.project_name)
    }
}

pub(crate) struct ServicesNamesRequest {
    pub(crate) project_name: String,
}

impl Request for ServicesNamesRequest {
    fn serialize(&self) -> String {
        format!("{}\n{}", "SERVICES-NAMES", self.project_name)
    }
}

pub(crate) struct ServiceInfoRequest {
    pub(crate) project_name: String,
    pub(crate) service_name: String,
}

impl Request for ServiceInfoRequest {
    fn serialize(&self) -> String {
        format!(
            "{}\n{}\n{}",
            "SERVICE-INFO", self.project_name, self.service_name
        )
    }
}

pub(crate) struct ServiceStartRequest {
    pub(crate) project_name: String,
    pub(crate) service_name: String,
}

impl Request for ServiceStartRequest {
    fn serialize(&self) -> String {
        format!(
            "{}\n{}\n{}",
            "SERVICE-START", self.project_name, self.service_name
        )
    }
}

pub(crate) struct ServiceStopRequest {
    pub(crate) project_name: String,
    pub(crate) service_name: String,
}

impl Request for ServiceStopRequest {
    fn serialize(&self) -> String {
        format!(
            "{}\n{}\n{}",
            "SERVICE-STOP", self.project_name, self.service_name
        )
    }
}
