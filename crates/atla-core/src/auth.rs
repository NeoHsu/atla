use serde::{Deserialize, Serialize};

const DEFAULT_SERVICE: &str = "atla";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRef {
    pub profile: String,
    pub email: String,
    pub instance: String,
}

impl CredentialRef {
    pub fn keyring_user(&self) -> String {
        format!("{}|{}|{}", self.profile, self.email, self.instance)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthState {
    Authenticated(CredentialRef),
    Missing,
}

pub trait CredentialStore {
    fn save_token(&self, credential: &CredentialRef, token: &str) -> Result<(), AuthError>;
    fn delete_token(&self, credential: &CredentialRef) -> Result<(), AuthError>;
    fn has_token(&self, credential: &CredentialRef) -> Result<bool, AuthError>;
}

#[derive(Debug, Clone)]
pub struct KeyringCredentialStore {
    service: String,
}

impl Default for KeyringCredentialStore {
    fn default() -> Self {
        Self::new(DEFAULT_SERVICE)
    }
}

impl KeyringCredentialStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry(&self, credential: &CredentialRef) -> Result<keyring::Entry, AuthError> {
        keyring::Entry::new(&self.service, &credential.keyring_user())
            .map_err(|error| AuthError::Backend(error.to_string()))
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn save_token(&self, credential: &CredentialRef, token: &str) -> Result<(), AuthError> {
        self.entry(credential)?
            .set_password(token)
            .map_err(|error| AuthError::Backend(error.to_string()))
    }

    fn delete_token(&self, credential: &CredentialRef) -> Result<(), AuthError> {
        match self.entry(credential)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(AuthError::Backend(error.to_string())),
        }
    }

    fn has_token(&self, credential: &CredentialRef) -> Result<bool, AuthError> {
        match self.entry(credential)?.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(error) => Err(AuthError::Backend(error.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("credential backend error: {0}")]
    Backend(String),
}
