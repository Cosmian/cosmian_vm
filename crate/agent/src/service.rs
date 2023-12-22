use crate::error::Error;

pub(crate) mod internal {
    use std::process::Command;

    use crate::error::Error;

    #[doc(hidden)]
    pub trait UnixServiceInternal {
        const NAME: &'static str;

        fn call(actions: &[&str]) -> Result<String, Error> {
            match Command::new(Self::NAME).args(actions).output() {
                Ok(output) => {
                    if output.status.success() {
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    } else {
                        Err(Error::Command(format!(
                            "Output: {} - error: {}",
                            String::from_utf8_lossy(&output.stdout).trim(),
                            String::from_utf8_lossy(&output.stderr).trim()
                        )))
                    }
                }
                Err(e) => Err(Error::Command(e.to_string())),
            }
        }
    }
}

pub trait UnixService: internal::UnixServiceInternal {
    fn start(service_app_name: &str) -> Result<String, Error> {
        Self::call(&["start", service_app_name])
    }

    fn stop(service_app_name: &str) -> Result<String, Error> {
        Self::call(&["stop", service_app_name])
    }

    fn reload(service_app_name: &str) -> Result<String, Error> {
        Self::call(&["reload", service_app_name])
    }
}

pub struct Supervisor;
impl UnixService for Supervisor {}
impl internal::UnixServiceInternal for Supervisor {
    const NAME: &'static str = "supervisorctl";
}
