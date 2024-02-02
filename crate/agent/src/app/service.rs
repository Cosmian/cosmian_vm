use crate::{error::Error, utils::call};
use serde::Deserialize;
use sysinfo::{ProcessExt, System, SystemExt};

pub trait UnixService {
    const NAME: &'static str;

    fn start(service_app_name: &str) -> Result<Option<String>, Error>;
    fn stop(service_app_name: &str) -> Result<Option<String>, Error>;
    fn restart(service_app_name: &str) -> Result<Option<String>, Error>;
}

pub struct Supervisor;
impl UnixService for Supervisor {
    const NAME: &'static str = "supervisorctl";

    fn start(service_app_name: &str) -> Result<Option<String>, Error> {
        call(Self::NAME, &["start", service_app_name], false)
    }

    fn stop(service_app_name: &str) -> Result<Option<String>, Error> {
        call(Self::NAME, &["stop", service_app_name], false)
    }

    fn restart(service_app_name: &str) -> Result<Option<String>, Error> {
        call(Self::NAME, &["restart", service_app_name], false)
    }
}

pub struct Systemd;
impl UnixService for Systemd {
    const NAME: &'static str = "system";

    fn start(service_app_name: &str) -> Result<Option<String>, Error> {
        call(Self::NAME, &[service_app_name, "start"], false)
    }

    fn stop(service_app_name: &str) -> Result<Option<String>, Error> {
        call(Self::NAME, &[service_app_name, "stop"], false)
    }

    fn restart(service_app_name: &str) -> Result<Option<String>, Error> {
        call(Self::NAME, &[service_app_name, "restart"], false)
    }
}

pub struct Standalone;
impl UnixService for Standalone {
    const NAME: &'static str = "";

    fn start(app_name: &str) -> Result<Option<String>, Error> {
        call(app_name, &[], true)
    }

    fn stop(app_name: &str) -> Result<Option<String>, Error> {
        let s = System::new_all();

        for process in s.processes_by_exact_name(app_name) {
            process.kill();
        }

        Ok(None)
    }

    fn restart(app_name: &str) -> Result<Option<String>, Error> {
        Self::stop(app_name)?;
        Self::start(app_name)
    }
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Supervisor,
    Systemd,
    Standalone,
}

impl ServiceType {
    pub fn start(&self, service_app_name: &str) -> Result<Option<String>, Error> {
        match self {
            Self::Supervisor => Supervisor::start(service_app_name),
            Self::Systemd => Systemd::start(service_app_name),
            Self::Standalone => Standalone::start(service_app_name),
        }
    }

    pub fn stop(&self, service_app_name: &str) -> Result<Option<String>, Error> {
        match self {
            Self::Supervisor => Supervisor::stop(service_app_name),
            Self::Systemd => Systemd::stop(service_app_name),
            Self::Standalone => Standalone::stop(service_app_name),
        }
    }

    pub fn reload(&self, service_app_name: &str) -> Result<Option<String>, Error> {
        match self {
            Self::Supervisor => Supervisor::restart(service_app_name),
            Self::Systemd => Systemd::restart(service_app_name),
            Self::Standalone => Standalone::restart(service_app_name),
        }
    }
}
