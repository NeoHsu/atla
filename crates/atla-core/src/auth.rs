use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

const DEFAULT_SERVICE: &str = "atla";
const ENV_TOKEN_KEYS: [&str; 2] = ["ATLA_TOKEN", "ATLA_API_TOKEN"];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialStorage {
    #[default]
    Keyring,
    File,
}

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
    fn get_token(&self, credential: &CredentialRef) -> Result<Option<String>, AuthError>;
    fn delete_token(&self, credential: &CredentialRef) -> Result<(), AuthError>;
    fn has_token(&self, credential: &CredentialRef) -> Result<bool, AuthError>;
}

pub fn env_token() -> Option<String> {
    ENV_TOKEN_KEYS
        .into_iter()
        .find_map(|key| env::var(key).ok().filter(|value| !value.is_empty()))
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

    fn get_token(&self, credential: &CredentialRef) -> Result<Option<String>, AuthError> {
        match self.entry(credential)?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(AuthError::Backend(error.to_string())),
        }
    }

    fn delete_token(&self, credential: &CredentialRef) -> Result<(), AuthError> {
        match self.entry(credential)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(AuthError::Backend(error.to_string())),
        }
    }

    fn has_token(&self, credential: &CredentialRef) -> Result<bool, AuthError> {
        self.get_token(credential).map(|token| token.is_some())
    }
}

#[derive(Debug, Clone)]
pub struct FileCredentialStore {
    path: PathBuf,
}

impl FileCredentialStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn default_path() -> Result<PathBuf, AuthError> {
        if let Some(path) = env::var_os("ATLA_CREDENTIALS") {
            return Ok(PathBuf::from(path));
        }

        let base_dirs = BaseDirs::new().ok_or(AuthError::ConfigDirUnavailable)?;
        Ok(base_dirs.config_dir().join("atla").join("credentials.toml"))
    }

    pub fn default_store() -> Result<Self, AuthError> {
        Ok(Self::new(Self::default_path()?))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn load(&self) -> Result<FileCredentials, AuthError> {
        if !self.path.exists() {
            return Ok(FileCredentials::default());
        }

        let contents = fs::read_to_string(&self.path)?;
        toml::from_str(&contents).map_err(|error| AuthError::Decode(error.to_string()))
    }

    fn save(&self, credentials: &FileCredentials) -> Result<(), AuthError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(credentials)
            .map_err(|error| AuthError::Encode(error.to_string()))?;
        fs::write(&self.path, contents)?;
        restrict_file_permissions(&self.path)?;
        Ok(())
    }
}

impl CredentialStore for FileCredentialStore {
    fn save_token(&self, credential: &CredentialRef, token: &str) -> Result<(), AuthError> {
        let mut credentials = self.load()?;
        credentials.tokens.insert(
            credential.keyring_user(),
            FileCredential {
                token: token.to_owned(),
            },
        );
        self.save(&credentials)
    }

    fn get_token(&self, credential: &CredentialRef) -> Result<Option<String>, AuthError> {
        Ok(self
            .load()?
            .tokens
            .get(&credential.keyring_user())
            .map(|credential| credential.token.clone()))
    }

    fn delete_token(&self, credential: &CredentialRef) -> Result<(), AuthError> {
        let mut credentials = self.load()?;
        credentials.tokens.remove(&credential.keyring_user());
        self.save(&credentials)
    }

    fn has_token(&self, credential: &CredentialRef) -> Result<bool, AuthError> {
        self.get_token(credential).map(|token| token.is_some())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct FileCredentials {
    #[serde(default)]
    tokens: BTreeMap<String, FileCredential>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileCredential {
    token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("could not find a user config directory")]
    ConfigDirUnavailable,
    #[error("credential backend error: {0}")]
    Backend(String),
    #[error("could not parse credentials: {0}")]
    Decode(String),
    #[error("could not encode credentials: {0}")]
    Encode(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(unix)]
fn restrict_file_permissions(path: &Path) -> Result<(), AuthError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &Path) -> Result<(), AuthError> {
    Ok(())
}
