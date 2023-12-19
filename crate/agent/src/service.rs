use crate::error::Error;
use serde::Deserialize;
use std::process::Command;
use sysinfo::{ProcessExt, System, SystemExt};

fn _call(exe: &str, args: &[&str]) -> Result<String, Error> {
    match Command::new(exe).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(Error::Command(format!(
                    "Output: {} - error: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )))
            }
        }
        Err(e) => Err(Error::Command(e.to_string())),
    }
}

pub trait UnixService {
    const NAME: &'static str;

    fn start(service_app_name: &str) -> Result<String, Error>;
    fn stop(service_app_name: &str) -> Result<String, Error>;
    fn reload(service_app_name: &str) -> Result<String, Error>;
}

pub struct Supervisor;
impl UnixService for Supervisor {
    const NAME: &'static str = "supervisorctl";

    fn start(service_app_name: &str) -> Result<String, Error> {
        _call(Self::NAME, &["start", service_app_name])
    }

    fn stop(service_app_name: &str) -> Result<String, Error> {
        _call(Self::NAME, &["stop", service_app_name])
    }

    fn reload(service_app_name: &str) -> Result<String, Error> {
        _call(Self::NAME, &["reload", service_app_name])
    }
}

pub struct Systemd;
impl UnixService for Systemd {
    const NAME: &'static str = "system";

    fn start(service_app_name: &str) -> Result<String, Error> {
        _call(Self::NAME, &[service_app_name, "start"])
    }

    fn stop(service_app_name: &str) -> Result<String, Error> {
        _call(Self::NAME, &[service_app_name, "stop"])
    }

    fn reload(service_app_name: &str) -> Result<String, Error> {
        _call(Self::NAME, &[service_app_name, "reload"])
    }
}

pub struct Standalone;
impl UnixService for Standalone {
    const NAME: &'static str = "";

    fn start(app_name: &str) -> Result<String, Error> {
        _call(app_name, &[])
    }

    fn stop(app_name: &str) -> Result<String, Error> {
        let s = System::new_all();

        for process in s.processes_by_exact_name(app_name) {
            process.kill();
        }

        Ok(String::from(""))
    }

    fn reload(app_name: &str) -> Result<String, Error> {
        Self::stop(app_name)?;
        Self::start(app_name)
    }
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Supervisor,
    Systemd,
    Standalone,
}

impl ServiceType {
    pub fn start(&self, service_app_name: &str) -> Result<String, Error> {
        match self {
            ServiceType::Supervisor => Supervisor::start(service_app_name),
            ServiceType::Systemd => Systemd::start(service_app_name),
            ServiceType::Standalone => Standalone::start(service_app_name),
        }
    }

    pub fn stop(&self, service_app_name: &str) -> Result<String, Error> {
        match self {
            ServiceType::Supervisor => Supervisor::stop(service_app_name),
            ServiceType::Systemd => Systemd::stop(service_app_name),
            ServiceType::Standalone => Standalone::stop(service_app_name),
        }
    }

    pub fn reload(&self, service_app_name: &str) -> Result<String, Error> {
        match self {
            ServiceType::Supervisor => Supervisor::reload(service_app_name),
            ServiceType::Systemd => Systemd::reload(service_app_name),
            ServiceType::Standalone => Standalone::reload(service_app_name),
        }
    }
}
