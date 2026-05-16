use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRef {
    pub profile: String,
    pub email: String,
    pub instance: String,
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

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("credential storage is not implemented yet")]
    NotImplemented,
}
