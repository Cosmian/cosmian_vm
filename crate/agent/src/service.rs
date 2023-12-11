use std::process::Output;

pub(crate) mod internal {
    use std::process::{Command, Output};

    #[doc(hidden)]
    pub trait UnixServiceInternal {
        const NAME: &'static str;

        fn call(actions: &[&str]) -> Result<Output, std::io::Error> {
            Command::new(Self::NAME).args(actions).output()
        }
    }
}

pub trait UnixService: internal::UnixServiceInternal {
    fn start(service_app_name: &str) -> Result<Output, std::io::Error> {
        Self::call(&[service_app_name, "start"])
    }

    fn stop(service_app_name: &str) -> Result<Output, std::io::Error> {
        Self::call(&[service_app_name, "stop"])
    }

    fn reload(service_app_name: &str) -> Result<Output, std::io::Error> {
        Self::call(&[service_app_name, "reload"])
    }
}

pub struct Supervisor;
impl UnixService for Supervisor {}
impl internal::UnixServiceInternal for Supervisor {
    const NAME: &'static str = "supervisorctl";
}
