use actix_http::body::MessageBody;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error as ActixError,
};
use actix_web_lab::middleware::Next;
use pep440::Version;

use crate::error::Error;
use cosmian_vm_client::client::USER_AGENT_ATTRIBUTE;

pub async fn check_user_agent_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, ActixError> {
    if let Err(error) = extract_and_check_user_agent(&req) {
        return Err(error.into());
    }

    next.call(req).await
}

/// This function will return an error only if the request
/// is from one of our clients and this client is outdated.
/// It will return `Ok` if the user agent is missing, empty,
/// incorrect or if the client is on a version with no security problem.
fn extract_and_check_user_agent(req: &ServiceRequest) -> Result<(), Error> {
    let user_agent = req.headers().get("user-agent");

    if let Some(user_agent) = user_agent {
        if let Ok(user_agent) = user_agent.to_str() {
            return check_user_agent(user_agent);
        }
    }

    Ok(())
}

fn check_user_agent(user_agent: &str) -> Result<(), Error> {
    _check_user_agent(user_agent, minimum_version())
}

fn _check_user_agent(user_agent: &str, minimum_version: Version) -> Result<(), Error> {
    match user_agent.rsplit_once('/') {
        Some((USER_AGENT_ATTRIBUTE, version_as_string)) => {
            let version = Version::parse(version_as_string);

            if let Some(mut version) = version {
                // We want to analyze the version 0.6a1 as 0.6 to compare below.
                version.pre = None;
                version.post = None;
                version.dev = None;
                version.local = vec![];

                if version < minimum_version {
                    return Err(Error::BadUserAgent(minimum_version.to_string()));
                }
            }

            Ok(())
        }
        _ => Ok(()),
    }
}

#[must_use]
pub fn minimum_version() -> Version {
    Version {
        epoch: 0,
        release: vec![1, 0],
        pre: None,
        post: None,
        dev: None,
        local: vec![],
    }
}

#[cfg(test)]
mod tests {
    use pep440::Version;

    use super::_check_user_agent;

    fn minimum_version() -> Version {
        Version {
            epoch: 0,
            release: vec![1, 2],
            pre: None,
            post: None,
            dev: None,
            local: vec![],
        }
    }

    #[test]
    fn test_check_user_agent() {
        let error_1_0 = _check_user_agent("cli-version/0.10", minimum_version());
        assert_eq!(
            error_1_0.unwrap_err().to_string(),
            "Please update the cosmian_vm cli to version 1.2"
        );

        assert!(_check_user_agent("cli-version/0.11", minimum_version()).is_err());
        assert!(_check_user_agent("cli-version/1.0", minimum_version()).is_err());
        assert!(_check_user_agent("cli-version/2.0", minimum_version()).is_ok());
        assert!(_check_user_agent("cli-version/1.2", minimum_version()).is_ok());
        assert!(_check_user_agent("cli-version/1.2.0", minimum_version()).is_ok());
        assert!(_check_user_agent("cli-version/1.2.1", minimum_version()).is_ok());
        assert!(_check_user_agent("cli-version/1.3.1", minimum_version()).is_ok());
        assert!(_check_user_agent("cli-version/1.2a1", minimum_version()).is_ok());

        assert!(_check_user_agent("bad_client/1.0", minimum_version()).is_ok());
        assert!(_check_user_agent("cli-version/bad_version", minimum_version()).is_ok());
        assert!(_check_user_agent("cli-version", minimum_version()).is_ok());
        assert!(_check_user_agent(
            "Mozilla/5.0 (X11; Linux x86_64; rv:101.0) Gecko/20100101 Firefox/101.0",
            minimum_version()
        )
        .is_ok());
    }
}
