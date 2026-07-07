//! Exit-code taxonomy and machine-readable error output.
//!
//! Runtime failures are classified so scripts and agents can branch on the
//! exit code instead of parsing stderr (clap itself exits 2 on usage errors):
//!
//! | Code | Kind        | Meaning                                              |
//! |------|-------------|------------------------------------------------------|
//! | 1    | `other`     | Any other failure (4xx business errors, IO, bugs)    |
//! | 2    | `usage`     | Invalid arguments (emitted by clap)                  |
//! | 3    | `auth`      | Missing/invalid credentials or profile (401/403)     |
//! | 4    | `not_found` | Resource does not exist (404)                        |
//! | 5    | `retryable` | Transient: network failure, 429, or 5xx              |

use atla_core::client::ApiError;

/// Marker for credential/profile setup failures so they classify as `auth`
/// even though they never reached the API.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AuthSetupError(pub String);

pub struct Classification {
    pub exit_code: i32,
    pub kind: &'static str,
    pub status: Option<u16>,
    pub retryable: bool,
}

impl Classification {
    fn new(exit_code: i32, kind: &'static str) -> Self {
        Self {
            exit_code,
            kind,
            status: None,
            retryable: false,
        }
    }
}

pub fn classify(err: &anyhow::Error) -> Classification {
    for cause in err.chain() {
        if cause.downcast_ref::<AuthSetupError>().is_some() {
            return Classification::new(3, "auth");
        }
        if let Some(api) = cause.downcast_ref::<ApiError>() {
            let status = api.status();
            let mut classification = match status.map(|s| s.as_u16()) {
                Some(401 | 403) => Classification::new(3, "auth"),
                Some(404) => Classification::new(4, "not_found"),
                _ if api.retryable() => Classification::new(5, "retryable"),
                _ => Classification::new(1, "other"),
            };
            classification.status = status.map(|s| s.as_u16());
            classification.retryable = api.retryable();
            return classification;
        }
    }
    Classification::new(1, "other")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn http_error(status: u16) -> anyhow::Error {
        anyhow::Error::new(ApiError::Http {
            status: reqwest::StatusCode::from_u16(status).unwrap(),
            body: "boom".to_owned(),
        })
        .context("failed to reach Jira")
    }

    #[test]
    fn classifies_auth_statuses() {
        assert_eq!(classify(&http_error(401)).exit_code, 3);
        assert_eq!(classify(&http_error(403)).exit_code, 3);
    }

    #[test]
    fn classifies_not_found() {
        let classification = classify(&http_error(404));
        assert_eq!(classification.exit_code, 4);
        assert_eq!(classification.kind, "not_found");
        assert_eq!(classification.status, Some(404));
    }

    #[test]
    fn classifies_retryable() {
        assert_eq!(classify(&http_error(429)).exit_code, 5);
        assert_eq!(classify(&http_error(503)).exit_code, 5);
        let network = anyhow::Error::new(ApiError::Network("connection reset".to_owned()));
        let classification = classify(&network);
        assert_eq!(classification.exit_code, 5);
        assert!(classification.retryable);
    }

    #[test]
    fn classifies_auth_setup() {
        let err = anyhow::Error::new(AuthSetupError("no active profile".to_owned()));
        assert_eq!(classify(&err).kind, "auth");
    }

    #[test]
    fn plain_errors_stay_generic() {
        assert_eq!(classify(&http_error(400)).exit_code, 1);
        assert_eq!(classify(&anyhow::anyhow!("boom")).exit_code, 1);
    }
}
