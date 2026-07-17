use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const DEFAULT_SERVICE: &str = "atla";
const ENV_TOKEN_KEYS: [&str; 2] = ["ATLA_TOKEN", "ATLA_API_TOKEN"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantDiscovery {
    pub site: String,
    pub cloud_id: String,
    pub jira_endpoint: String,
    pub confluence_endpoint: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TenantInfoResponse {
    cloud_id: String,
}

pub async fn discover_tenant(
    site: &str,
    policy: crate::HttpPolicy,
) -> Result<TenantDiscovery, crate::client::ApiError> {
    let mut site_url = reqwest::Url::parse(site)
        .map_err(|error| crate::client::ApiError::Decode(format!("invalid site URL: {error}")))?;
    if !matches!(site_url.scheme(), "http" | "https")
        || site_url.host_str().is_none()
        || !site_url.username().is_empty()
        || site_url.password().is_some()
    {
        return Err(crate::client::ApiError::Decode(
            "site URL must be an HTTP(S) origin without embedded credentials".to_owned(),
        ));
    }
    site_url.set_path("");
    site_url.set_query(None);
    site_url.set_fragment(None);
    let site = site_url.as_str().trim_end_matches('/').to_owned();
    let tenant_info_url = site_url
        .join("/_edge/tenant_info")
        .map_err(|error| crate::client::ApiError::Decode(error.to_string()))?;
    let client = reqwest::Client::builder()
        .connect_timeout(policy.connect_timeout)
        .timeout(policy.request_timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let response = client.get(tenant_info_url).send().await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(crate::client::ApiError::Http {
            status,
            body: crate::client::extract_api_error_body(&body),
        });
    }
    let tenant: TenantInfoResponse = serde_json::from_str(&body).map_err(|error| {
        crate::client::ApiError::Decode(format!("invalid tenant-info response: {error}"))
    })?;
    let cloud_id = crate::profile::normalize_cloud_id(&tenant.cloud_id)
        .map_err(|error| crate::client::ApiError::Decode(error.to_string()))?
        .ok_or_else(|| {
            crate::client::ApiError::Decode("tenant-info cloudId is empty".to_owned())
        })?;

    Ok(TenantDiscovery {
        site,
        jira_endpoint: format!("https://api.atlassian.com/ex/jira/{cloud_id}"),
        confluence_endpoint: format!("https://api.atlassian.com/ex/confluence/{cloud_id}"),
        cloud_id,
    })
}

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

        Ok(xdg_config_dir()
            .ok_or(AuthError::ConfigDirUnavailable)?
            .join("atla")
            .join("credentials.toml"))
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
        let contents = toml::to_string_pretty(credentials)
            .map_err(|error| AuthError::Encode(error.to_string()))?;
        crate::secure_file::atomic_write(&self.path, contents.as_bytes())?;
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

fn xdg_config_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
            return Some(PathBuf::from(xdg));
        }
        if let Some(home) = env::var_os("HOME").filter(|v| !v.is_empty()) {
            return Some(PathBuf::from(home).join(".config"));
        }
        home_dir_from_passwd().map(|h| h.join(".config"))
    }
    #[cfg(not(unix))]
    {
        directories::BaseDirs::new().map(|d| d.config_dir().to_path_buf())
    }
}

#[cfg(unix)]
fn home_dir_from_passwd() -> Option<PathBuf> {
    use std::ffi::CStr;
    // Safety: getpwuid(getuid()) returns a pointer valid until the next
    // getpwent/getpwuid call on this thread; we copy the path immediately.
    unsafe {
        let pw = libc::getpwuid(libc::getuid());
        if pw.is_null() {
            return None;
        }
        if (*pw).pw_dir.is_null() {
            return None;
        }
        CStr::from_ptr((*pw).pw_dir)
            .to_str()
            .ok()
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_store_round_trips_token_with_restricted_permissions() {
        let directory = tempfile::tempdir().expect("temp directory");
        let store = FileCredentialStore::new(directory.path().join("credentials.toml"));
        let credential = CredentialRef {
            profile: "work".to_owned(),
            email: "neo@example.com".to_owned(),
            instance: "https://example.atlassian.net".to_owned(),
        };

        store
            .save_token(&credential, "test-token")
            .expect("save token");

        assert_eq!(
            store.get_token(&credential).expect("load token").as_deref(),
            Some("test-token")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = fs::metadata(store.path())
                .expect("credential metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        store.delete_token(&credential).expect("delete token");
        assert_eq!(store.get_token(&credential).expect("load token"), None);
    }

    #[tokio::test]
    async fn discovers_cloud_id_and_product_endpoints() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/_edge/tenant_info"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"cloudId": "cloud-123"})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let discovery = discover_tenant(&server.uri(), crate::HttpPolicy::default())
            .await
            .expect("discover tenant");

        assert_eq!(discovery.cloud_id, "cloud-123");
        assert_eq!(
            discovery.jira_endpoint,
            "https://api.atlassian.com/ex/jira/cloud-123"
        );
        assert_eq!(
            discovery.confluence_endpoint,
            "https://api.atlassian.com/ex/confluence/cloud-123"
        );
        server.verify().await;
    }
}
