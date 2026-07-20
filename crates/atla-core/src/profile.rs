use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{CredentialRef, CredentialStorage};

pub const CURRENT_CONFIG_SCHEMA_VERSION: u32 = 2;
const LEGACY_CONFIG_SCHEMA_VERSION: u32 = 1;

const fn legacy_config_schema_version() -> u32 {
    LEGACY_CONFIG_SCHEMA_VERSION
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlaConfig {
    #[serde(default = "legacy_config_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub default: DefaultSection,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
}

impl Default for AtlaConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_CONFIG_SCHEMA_VERSION,
            default: DefaultSection::default(),
            profiles: BTreeMap::new(),
            aliases: BTreeMap::new(),
        }
    }
}

impl AtlaConfig {
    pub fn active_profile_name<'a>(&'a self, override_name: Option<&'a str>) -> Option<&'a str> {
        override_name
            .or(self.default.profile.as_deref())
            .or_else(|| self.profiles.keys().next().map(String::as_str))
    }

    pub fn active_profile(&self, override_name: Option<&str>) -> Option<(&str, &Profile)> {
        let requested_name = self.active_profile_name(override_name)?;
        let (name, profile) = self.profiles.get_key_value(requested_name)?;
        Some((name.as_str(), profile))
    }

    pub fn upsert_profile(&mut self, name: impl Into<String>, mut profile: Profile) {
        let name = name.into();

        if let Some(existing) = self.profiles.get(&name) {
            profile.default_project = profile.default_project.or(existing.default_project.clone());
            profile.default_space = profile.default_space.or(existing.default_space.clone());
            profile.cloud_id = profile.cloud_id.or(existing.cloud_id.clone());
            if profile.policy.is_default() {
                profile.policy = existing.policy.clone();
            }
        }

        self.profiles.insert(name.clone(), profile);
        if self.default.profile.is_none() {
            self.default.profile = Some(name);
        }
    }

    pub fn switch_profile(&mut self, name: &str) -> Result<(), ProfileError> {
        if !self.profiles.contains_key(name) {
            return Err(ProfileError::MissingProfile(name.to_owned()));
        }

        self.default.profile = Some(name.to_owned());
        Ok(())
    }

    pub fn set_value(
        &mut self,
        key: &str,
        value: String,
        profile_name: Option<&str>,
    ) -> Result<(), ProfileError> {
        let key = normalized_key(key);
        if let Some(alias) = key
            .strip_prefix("alias.")
            .or_else(|| key.strip_prefix("aliases."))
        {
            self.aliases.insert(alias.to_owned(), value);
            return Ok(());
        }

        if key == "default.profile" {
            return self.switch_profile(&value);
        }

        if let Some(rest) = key.strip_prefix("profiles.") {
            if let Some((profile_name, field)) = rest.rsplit_once(".policy.") {
                let profile = self
                    .profiles
                    .get_mut(profile_name)
                    .ok_or_else(|| ProfileError::MissingProfile(profile_name.to_owned()))?;
                match normalized_key(field).as_str() {
                    "mode" => profile.policy.mode = parse_policy_mode(&value)?,
                    "allow" => profile.policy.allow = parse_policy_patterns(&value)?,
                    "deny" => profile.policy.deny = parse_policy_patterns(&value)?,
                    _ => {
                        return Err(ProfileError::UnsupportedConfigKey(format!(
                            "profiles.{profile_name}.policy.{field}"
                        )));
                    }
                }
                return Ok(());
            }
            if let Some(dot_pos) = rest.rfind('.') {
                let profile_name = &rest[..dot_pos];
                let field = normalized_key(&rest[dot_pos + 1..]);
                let profile = self
                    .profiles
                    .get_mut(profile_name)
                    .ok_or_else(|| ProfileError::MissingProfile(profile_name.to_owned()))?;
                match field.as_str() {
                    "instance" => profile.instance = value,
                    "email" => profile.email = value,
                    "credential-store" => {
                        profile.credential_store = parse_credential_storage(&value)?;
                    }
                    "default-project" => profile.default_project = Some(value),
                    "default-space" => profile.default_space = Some(value),
                    "cloud-id" => profile.cloud_id = normalize_cloud_id(&value)?,
                    _ => {
                        return Err(ProfileError::UnsupportedConfigKey(format!(
                            "profiles.{profile_name}.{field}"
                        )));
                    }
                }
                return Ok(());
            }
            return Err(ProfileError::UnsupportedConfigKey(key.to_owned()));
        }

        match key.as_str() {
            "default-profile" => self.switch_profile(&value),
            "default-project" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.default_project = Some(value);
                Ok(())
            }
            "default-space" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.default_space = Some(value);
                Ok(())
            }
            "instance" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.instance = value;
                Ok(())
            }
            "email" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.email = value;
                Ok(())
            }
            "credential-store" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.credential_store = parse_credential_storage(&value)?;
                Ok(())
            }
            "cloud-id" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.cloud_id = normalize_cloud_id(&value)?;
                Ok(())
            }
            "policy-mode" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.policy.mode = parse_policy_mode(&value)?;
                Ok(())
            }
            "policy-allow" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.policy.allow = parse_policy_patterns(&value)?;
                Ok(())
            }
            "policy-deny" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.policy.deny = parse_policy_patterns(&value)?;
                Ok(())
            }
            _ => Err(ProfileError::UnsupportedConfigKey(key.to_owned())),
        }
    }

    pub fn get_value(
        &self,
        key: &str,
        profile_name: Option<&str>,
    ) -> Result<Option<String>, ProfileError> {
        let key = normalized_key(key);
        if let Some(alias) = key
            .strip_prefix("alias.")
            .or_else(|| key.strip_prefix("aliases."))
        {
            return Ok(self.aliases.get(alias).cloned());
        }

        if key == "default.profile" {
            return Ok(self.default.profile.clone());
        }
        if key == "schema-version" {
            return Ok(Some(self.schema_version.to_string()));
        }

        if let Some(rest) = key.strip_prefix("profiles.") {
            if let Some((profile_name, field)) = rest.rsplit_once(".policy.") {
                let profile = self
                    .profiles
                    .get(profile_name)
                    .ok_or_else(|| ProfileError::MissingProfile(profile_name.to_owned()))?;
                let value = match normalized_key(field).as_str() {
                    "mode" => Some(profile.policy.mode.to_string()),
                    "allow" => Some(profile.policy.allow.join(",")),
                    "deny" => Some(profile.policy.deny.join(",")),
                    _ => {
                        return Err(ProfileError::UnsupportedConfigKey(format!(
                            "profiles.{profile_name}.policy.{field}"
                        )));
                    }
                };
                return Ok(value);
            }
            if let Some(dot_pos) = rest.rfind('.') {
                let profile_name = &rest[..dot_pos];
                let field = normalized_key(&rest[dot_pos + 1..]);
                let profile = self
                    .profiles
                    .get(profile_name)
                    .ok_or_else(|| ProfileError::MissingProfile(profile_name.to_owned()))?;
                let value = match field.as_str() {
                    "instance" => Some(profile.instance.clone()),
                    "email" => Some(profile.email.clone()),
                    "credential-store" => Some(profile.credential_store.to_string()),
                    "default-project" => profile.default_project.clone(),
                    "default-space" => profile.default_space.clone(),
                    "cloud-id" => profile.cloud_id.clone(),
                    _ => {
                        return Err(ProfileError::UnsupportedConfigKey(format!(
                            "profiles.{profile_name}.{field}"
                        )));
                    }
                };
                return Ok(value);
            }
            return Err(ProfileError::UnsupportedConfigKey(key.to_owned()));
        }

        let value = match key.as_str() {
            "default-profile" => self.default.profile.clone(),
            "default-project" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                profile.default_project.clone()
            }
            "default-space" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                profile.default_space.clone()
            }
            "instance" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.instance.clone())
            }
            "email" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.email.clone())
            }
            "credential-store" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.credential_store.to_string())
            }
            "cloud-id" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                profile.cloud_id.clone()
            }
            "policy-mode" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.policy.mode.to_string())
            }
            "policy-allow" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.policy.allow.join(","))
            }
            "policy-deny" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.policy.deny.join(","))
            }
            _ => return Err(ProfileError::UnsupportedConfigKey(key.to_owned())),
        };

        Ok(value)
    }

    fn active_profile_mut(
        &mut self,
        override_name: Option<&str>,
    ) -> Result<&mut Profile, ProfileError> {
        let name = self
            .active_profile_name(override_name)
            .ok_or(ProfileError::MissingActiveProfile)?
            .to_owned();

        self.profiles
            .get_mut(&name)
            .ok_or(ProfileError::MissingProfile(name))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultSection {
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlassianProduct {
    Jira,
    Confluence,
}

impl AtlassianProduct {
    fn gateway_segment(self) -> &'static str {
        match self {
            Self::Jira => "jira",
            Self::Confluence => "confluence",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyMode {
    ReadOnly,
    #[default]
    ReadWrite,
}

impl std::fmt::Display for PolicyMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => formatter.write_str("read-only"),
            Self::ReadWrite => formatter.write_str("read-write"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecisionSource {
    Deny,
    Allow,
    Mode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePolicyDecision {
    pub allowed: bool,
    pub source: PolicyDecisionSource,
    pub matched_pattern: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilePolicy {
    #[serde(default)]
    pub mode: PolicyMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
}

impl ProfilePolicy {
    pub fn is_default(&self) -> bool {
        self.mode == PolicyMode::ReadWrite && self.allow.is_empty() && self.deny.is_empty()
    }

    pub fn decision(&self, operation_id: &str, mutates: bool) -> ProfilePolicyDecision {
        if let Some(pattern) = self
            .deny
            .iter()
            .find(|pattern| wildcard_matches(pattern, operation_id))
        {
            return ProfilePolicyDecision {
                allowed: false,
                source: PolicyDecisionSource::Deny,
                matched_pattern: Some(pattern.clone()),
            };
        }
        if let Some(pattern) = self
            .allow
            .iter()
            .find(|pattern| wildcard_matches(pattern, operation_id))
        {
            return ProfilePolicyDecision {
                allowed: true,
                source: PolicyDecisionSource::Allow,
                matched_pattern: Some(pattern.clone()),
            };
        }
        ProfilePolicyDecision {
            allowed: self.mode == PolicyMode::ReadWrite || !mutates,
            source: PolicyDecisionSource::Mode,
            matched_pattern: None,
        }
    }

    pub fn allows(&self, operation_id: &str, mutates: bool) -> bool {
        self.decision(operation_id, mutates).allowed
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub instance: String,
    pub email: String,
    #[serde(default)]
    pub credential_store: CredentialStorage,
    pub default_project: Option<String>,
    pub default_space: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cloud_id: Option<String>,
    #[serde(default, skip_serializing_if = "ProfilePolicy::is_default")]
    pub policy: ProfilePolicy,
}

impl Profile {
    pub fn credential_ref(&self, name: impl Into<String>) -> CredentialRef {
        CredentialRef {
            profile: name.into(),
            email: self.email.clone(),
            instance: self.instance.clone(),
        }
    }

    /// Product API root. Profiles without a cloud ID use the site URL;
    /// scoped-token profiles route through Atlassian's product gateway.
    pub fn api_base_url(&self, product: AtlassianProduct) -> String {
        match &self.cloud_id {
            Some(cloud_id) => format!(
                "https://api.atlassian.com/ex/{}/{cloud_id}",
                product.gateway_segment()
            ),
            None => self.instance.trim_end_matches('/').to_owned(),
        }
    }

    pub fn jira_api_base_url(&self) -> String {
        self.api_base_url(AtlassianProduct::Jira)
    }

    pub fn confluence_api_base_url(&self) -> String {
        self.api_base_url(AtlassianProduct::Confluence)
    }

    pub fn uses_scoped_token_gateway(&self) -> bool {
        self.cloud_id.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn default_path() -> Result<PathBuf, ProfileError> {
        if let Some(path) = env::var_os("ATLA_CONFIG") {
            return Ok(PathBuf::from(path));
        }

        Ok(xdg_config_dir()
            .ok_or(ProfileError::ConfigDirUnavailable)?
            .join("atla")
            .join("config.toml"))
    }

    pub fn default_store() -> Result<Self, ProfileError> {
        Ok(Self::new(Self::default_path()?))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<AtlaConfig, ProfileError> {
        self.load_with_migration(true)
    }

    /// Loads and upgrades legacy data in memory without modifying the file.
    /// Used by the CLI's strict read-only execution policy.
    pub fn load_read_only(&self) -> Result<AtlaConfig, ProfileError> {
        self.load_with_migration(false)
    }

    fn load_with_migration(&self, persist_migration: bool) -> Result<AtlaConfig, ProfileError> {
        if !self.path.exists() {
            return Ok(AtlaConfig::default());
        }

        let contents = fs::read_to_string(&self.path)?;
        let mut config: AtlaConfig =
            toml::from_str(&contents).map_err(|error| ProfileError::Decode(error.to_string()))?;
        if config.schema_version > CURRENT_CONFIG_SCHEMA_VERSION
            || config.schema_version < LEGACY_CONFIG_SCHEMA_VERSION
        {
            return Err(ProfileError::UnsupportedConfigVersion {
                found: config.schema_version,
                supported: CURRENT_CONFIG_SCHEMA_VERSION,
            });
        }
        if config.schema_version < CURRENT_CONFIG_SCHEMA_VERSION {
            let previous_version = config.schema_version;
            config.schema_version = CURRENT_CONFIG_SCHEMA_VERSION;
            if persist_migration {
                let backup = migration_backup_path(&self.path, previous_version);
                if !backup.exists() {
                    crate::secure_file::atomic_write(&backup, contents.as_bytes())?;
                }
                self.save(&config)?;
            }
        }
        Ok(config)
    }

    pub fn save(&self, config: &AtlaConfig) -> Result<(), ProfileError> {
        let contents = toml::to_string_pretty(config)
            .map_err(|error| ProfileError::Encode(error.to_string()))?;
        crate::secure_file::atomic_write(&self.path, contents.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("could not find a user config directory")]
    ConfigDirUnavailable,
    #[error("profile `{0}` does not exist")]
    MissingProfile(String),
    #[error("no active profile is configured")]
    MissingActiveProfile,
    #[error("unsupported config key `{0}`")]
    UnsupportedConfigKey(String),
    #[error("unsupported credential store `{0}`; expected `keyring` or `file`")]
    UnsupportedCredentialStore(String),
    #[error(
        "unsupported config schema version {found}; this atla build supports version {supported}"
    )]
    UnsupportedConfigVersion { found: u32, supported: u32 },
    #[error(
        "invalid cloud ID `{0}`; expected an Atlassian cloud ID containing letters, digits, `_`, or `-`"
    )]
    InvalidCloudId(String),
    #[error("invalid policy mode `{0}`; expected `read-only` or `read-write`")]
    InvalidPolicyMode(String),
    #[error("invalid operation policy pattern `{0}`")]
    InvalidPolicyPattern(String),
    #[error("could not parse config: {0}")]
    Decode(String),
    #[error("could not encode config: {0}")]
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

fn migration_backup_path(path: &Path, from_version: u32) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config.toml");
    path.with_file_name(format!("{file_name}.v{from_version}.bak"))
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut previous = vec![false; value.len() + 1];
    previous[0] = true;

    for token in pattern {
        let mut current = vec![false; value.len() + 1];
        if *token == b'*' {
            current[0] = previous[0];
            for index in 1..=value.len() {
                current[index] = previous[index] || current[index - 1];
            }
        } else {
            for index in 1..=value.len() {
                current[index] = previous[index - 1] && *token == value[index - 1];
            }
        }
        previous = current;
    }
    previous[value.len()]
}

fn parse_policy_patterns(value: &str) -> Result<Vec<String>, ProfileError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(|pattern| {
            if pattern.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'*')
            }) {
                Ok(pattern.to_owned())
            } else {
                Err(ProfileError::InvalidPolicyPattern(pattern.to_owned()))
            }
        })
        .collect()
}

fn parse_policy_mode(value: &str) -> Result<PolicyMode, ProfileError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "read-only" | "readonly" => Ok(PolicyMode::ReadOnly),
        "read-write" | "readwrite" => Ok(PolicyMode::ReadWrite),
        _ => Err(ProfileError::InvalidPolicyMode(value.to_owned())),
    }
}

pub fn normalize_cloud_id(value: &str) -> Result<Option<String>, ProfileError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        Ok(Some(value.to_owned()))
    } else {
        Err(ProfileError::InvalidCloudId(value.to_owned()))
    }
}

fn normalized_key(key: &str) -> String {
    key.replace('_', "-").to_ascii_lowercase()
}

fn parse_credential_storage(value: &str) -> Result<CredentialStorage, ProfileError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "keyring" => Ok(CredentialStorage::Keyring),
        "file" => Ok(CredentialStorage::File),
        _ => Err(ProfileError::UnsupportedCredentialStore(value.to_owned())),
    }
}

impl std::fmt::Display for CredentialStorage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keyring => formatter.write_str("keyring"),
            Self::File => formatter.write_str("file"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = ConfigStore::new(dir.path().join("config.toml"));
        let mut config = AtlaConfig::default();

        config.upsert_profile(
            "work",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: Some("PROJ".to_owned()),
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        store.save(&config).expect("save config");
        let loaded = store.load().expect("load config");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = fs::metadata(store.path())
                .expect("config metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
        assert_eq!(loaded.default.profile.as_deref(), Some("work"));
        assert_eq!(
            loaded.profiles["work"].default_project.as_deref(),
            Some("PROJ")
        );
    }

    #[test]
    fn migrates_legacy_config_and_preserves_a_backup() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("config.toml");
        let store = ConfigStore::new(&path);
        let legacy = r#"
[default]
profile = "work"

[profiles.work]
instance = "https://example.atlassian.net"
email = "neo@example.com"
"#;
        fs::write(&path, legacy).expect("write legacy config");

        let config = store.load().expect("migrate config");

        assert_eq!(config.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
        assert_eq!(
            fs::read_to_string(directory.path().join("config.toml.v1.bak"))
                .expect("migration backup"),
            legacy
        );
        let migrated = fs::read_to_string(&path).expect("migrated config");
        assert!(migrated.contains("schema_version = 2"));
    }

    #[test]
    fn read_only_load_upgrades_in_memory_without_writing() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("config.toml");
        let legacy = "[profiles.work]\ninstance = \"https://example.atlassian.net\"\nemail = \"neo@example.com\"\n";
        fs::write(&path, legacy).expect("write legacy config");
        let store = ConfigStore::new(&path);

        let config = store.load_read_only().expect("read-only load");

        assert_eq!(config.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
        assert_eq!(fs::read_to_string(&path).expect("unchanged config"), legacy);
        assert!(!directory.path().join("config.toml.v1.bak").exists());
    }

    #[test]
    fn rejects_newer_config_schema() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("config.toml");
        fs::write(&path, "schema_version = 99\n").expect("write future config");
        let store = ConfigStore::new(path);

        let error = store.load().expect_err("future schema should fail");

        assert!(matches!(
            error,
            ProfileError::UnsupportedConfigVersion {
                found: 99,
                supported: CURRENT_CONFIG_SCHEMA_VERSION,
            }
        ));
    }

    #[test]
    fn scoped_profile_builds_product_specific_gateway_urls() {
        let profile = Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: Some("cloud-123".to_owned()),
            policy: ProfilePolicy::default(),
        };

        assert_eq!(
            profile.api_base_url(AtlassianProduct::Jira),
            "https://api.atlassian.com/ex/jira/cloud-123"
        );
        assert_eq!(
            profile.api_base_url(AtlassianProduct::Confluence),
            "https://api.atlassian.com/ex/confluence/cloud-123"
        );
    }

    #[test]
    fn cloud_id_config_can_be_set_and_cleared() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        config
            .set_value("cloud-id", "cloud-123".to_owned(), Some("work"))
            .expect("set cloud ID");
        assert_eq!(
            config.get_value("cloud-id", Some("work")).expect("get"),
            Some("cloud-123".to_owned())
        );
        config
            .set_value("cloud-id", String::new(), Some("work"))
            .expect("clear cloud ID");
        assert_eq!(
            config.get_value("cloud-id", Some("work")).expect("get"),
            None
        );
    }

    #[test]
    fn operation_policy_applies_deny_allow_then_mode() {
        let mut policy = ProfilePolicy {
            mode: PolicyMode::ReadOnly,
            allow: vec!["jira.issue.comment.add".to_owned()],
            deny: vec!["*.delete".to_owned()],
        };

        assert!(policy.allows("jira.issue.view", false));
        assert!(policy.allows("jira.issue.comment.add", true));
        assert!(!policy.allows("jira.issue.create", true));
        assert!(!policy.allows("jira.issue.delete", true));
        assert_eq!(
            policy.decision("jira.issue.create", true),
            ProfilePolicyDecision {
                allowed: false,
                source: PolicyDecisionSource::Mode,
                matched_pattern: None,
            }
        );

        policy.allow.push("jira.issue.delete".to_owned());
        assert!(!policy.allows("jira.issue.delete", true));
        assert_eq!(
            policy.decision("jira.issue.delete", true),
            ProfilePolicyDecision {
                allowed: false,
                source: PolicyDecisionSource::Deny,
                matched_pattern: Some("*.delete".to_owned()),
            }
        );
    }

    #[test]
    fn policy_config_keys_round_trip() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        config
            .set_value("profiles.work.policy.mode", "read-only".to_owned(), None)
            .expect("set mode");
        config
            .set_value(
                "profiles.work.policy.allow",
                "jira.issue.view, jira.issue.comment.add".to_owned(),
                None,
            )
            .expect("set allow");
        config
            .set_value("profiles.work.policy.deny", "*.delete".to_owned(), None)
            .expect("set deny");

        assert_eq!(
            config
                .get_value("profiles.work.policy.mode", None)
                .expect("get mode")
                .as_deref(),
            Some("read-only")
        );
        assert_eq!(
            config
                .get_value("profiles.work.policy.allow", None)
                .expect("get allow")
                .as_deref(),
            Some("jira.issue.view,jira.issue.comment.add")
        );
        assert!(
            !config.profiles["work"]
                .policy
                .allows("confluence.page.delete", true)
        );
    }

    #[test]
    fn switches_to_existing_profile_only() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        config.switch_profile("work").expect("switch profile");
        assert!(matches!(
            config.switch_profile("missing"),
            Err(ProfileError::MissingProfile(name)) if name == "missing"
        ));
    }

    #[test]
    fn get_value_with_dotted_keys() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "default",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: Some("PROJ".to_owned()),
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        assert_eq!(
            config
                .get_value("default.profile", None)
                .expect("default profile value")
                .as_deref(),
            Some("default")
        );
        assert_eq!(
            config
                .get_value("profiles.default.instance", None)
                .expect("profile instance value")
                .as_deref(),
            Some("https://example.atlassian.net")
        );
        assert_eq!(
            config
                .get_value("profiles.default.email", None)
                .expect("profile email value")
                .as_deref(),
            Some("neo@example.com")
        );
    }

    #[test]
    fn set_value_with_dotted_keys() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "default",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        config
            .set_value(
                "profiles.default.instance",
                "https://new.atlassian.net".to_owned(),
                None,
            )
            .expect("set profile instance");
        assert_eq!(
            config
                .get_value("profiles.default.instance", None)
                .expect("updated profile instance")
                .as_deref(),
            Some("https://new.atlassian.net")
        );
    }

    #[test]
    fn get_value_unsupported_key_errors() {
        let config = AtlaConfig::default();
        assert!(matches!(
            config.get_value("nonexistent.key", None),
            Err(ProfileError::UnsupportedConfigKey(_))
        ));
    }

    #[test]
    fn sets_and_gets_aliases() {
        let mut config = AtlaConfig::default();

        config
            .set_value(
                "alias.mine",
                "jira search 'assignee = currentUser()'".to_owned(),
                None,
            )
            .expect("set alias");

        assert_eq!(
            config
                .get_value("aliases.mine", None)
                .expect("get alias")
                .as_deref(),
            Some("jira search 'assignee = currentUser()'")
        );
    }

    #[test]
    fn active_profile_returns_none_when_empty() {
        let config = AtlaConfig::default();
        assert!(config.active_profile(None).is_none());
    }

    #[test]
    fn active_profile_returns_default_profile() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        let (name, profile) = config.active_profile(None).expect("active profile");
        assert_eq!(name, "work");
        assert_eq!(profile.instance, "https://work.atlassian.net");
        assert_eq!(profile.email, "work@example.com");
    }

    #[test]
    fn active_profile_respects_override_name() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );
        config.upsert_profile(
            "personal",
            Profile {
                instance: "https://personal.atlassian.net".to_owned(),
                email: "personal@example.com".to_owned(),
                credential_store: CredentialStorage::File,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        let (name, profile) = config
            .active_profile(Some("personal"))
            .expect("active profile with override");
        assert_eq!(name, "personal");
        assert_eq!(profile.instance, "https://personal.atlassian.net");
    }

    #[test]
    fn active_profile_override_takes_precedence_over_default() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "default",
            Profile {
                instance: "https://default.atlassian.net".to_owned(),
                email: "default@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::File,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        let (name, profile) = config
            .active_profile(Some("work"))
            .expect("active profile with override");
        assert_eq!(name, "work");
        assert_eq!(profile.instance, "https://work.atlassian.net");
    }

    #[test]
    fn active_profile_name_with_no_default_returns_none() {
        let config = AtlaConfig::default();
        assert_eq!(config.active_profile_name(None), None);
    }

    #[test]
    fn active_profile_name_with_override() {
        let config = AtlaConfig::default();
        assert_eq!(config.active_profile_name(Some("work")), Some("work"));
    }

    #[test]
    fn get_value_default_project() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "default",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: Some("PROJ".to_owned()),
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        assert_eq!(
            config
                .get_value("profiles.default.default_project", None)
                .expect("get default project")
                .as_deref(),
            Some("PROJ")
        );
    }

    #[test]
    fn get_value_default_space() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "default",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: Some("DEV".to_owned()),
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        assert_eq!(
            config
                .get_value("profiles.default.default_space", None)
                .expect("get default space")
                .as_deref(),
            Some("DEV")
        );
    }

    #[test]
    fn get_value_credential_store() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "default",
            Profile {
                instance: "https://example.atlassian.net".to_owned(),
                email: "neo@example.com".to_owned(),
                credential_store: CredentialStorage::File,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        assert_eq!(
            config
                .get_value("profiles.default.credential_store", None)
                .expect("get credential store")
                .as_deref(),
            Some("file")
        );
    }

    #[test]
    fn get_value_missing_profile_key_returns_none() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        assert_eq!(
            config
                .get_value("profiles.work.default_project", None)
                .expect("get missing optional key"),
            None
        );
    }

    #[test]
    fn get_value_nonexistent_profile_key_errors() {
        let config = AtlaConfig::default();
        assert!(matches!(
            config.get_value("profiles.nonexistent.instance", None),
            Err(ProfileError::MissingProfile(name)) if name == "nonexistent"
        ));
    }

    #[test]
    fn upsert_second_profile_does_not_change_default() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );
        config.upsert_profile(
            "personal",
            Profile {
                instance: "https://personal.atlassian.net".to_owned(),
                email: "personal@example.com".to_owned(),
                credential_store: CredentialStorage::File,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        assert_eq!(config.active_profile_name(None), Some("work"));
    }

    #[test]
    fn config_store_loads_empty_file_as_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = ConfigStore::new(dir.path().join("config.toml"));

        let loaded = store.load().expect("load missing config as default");
        assert_eq!(loaded, AtlaConfig::default());
    }

    #[test]
    fn config_stores_and_loads_aliases() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = ConfigStore::new(dir.path().join("config.toml"));
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );
        config
            .set_value(
                "alias.deploy",
                "jira search 'project = DEP'".to_owned(),
                None,
            )
            .expect("set alias");

        store.save(&config).expect("save config");
        let loaded = store.load().expect("load config");

        assert_eq!(
            loaded.aliases.get("deploy"),
            Some(&"jira search 'project = DEP'".to_owned())
        );
    }

    #[test]
    fn profile_without_optional_fields_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = ConfigStore::new(dir.path().join("config.toml"));
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::File,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        store.save(&config).expect("save config");
        let loaded = store.load().expect("load config");

        assert_eq!(loaded.profiles["work"].default_project, None);
        assert_eq!(loaded.profiles["work"].default_space, None);
    }

    #[test]
    fn set_value_for_specific_profile_with_override() {
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );
        config.upsert_profile(
            "personal",
            Profile {
                instance: "https://personal.atlassian.net".to_owned(),
                email: "personal@example.com".to_owned(),
                credential_store: CredentialStorage::File,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        config
            .set_value("profiles.work.default_project", "WORK".to_owned(), None)
            .expect("set work default project");

        assert_eq!(
            config.profiles["work"].default_project.as_deref(),
            Some("WORK")
        );
        assert_eq!(config.profiles["personal"].default_project, None);
    }

    #[test]
    fn get_value_alias_name() {
        let mut config = AtlaConfig::default();
        config
            .set_value(
                "alias.deploy",
                "jira search 'project = DEP'".to_owned(),
                None,
            )
            .expect("set alias");

        assert_eq!(
            config
                .get_value("aliases.deploy", None)
                .expect("get alias")
                .as_deref(),
            Some("jira search 'project = DEP'")
        );
    }

    #[test]
    fn multiple_profiles_coexist() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = ConfigStore::new(dir.path().join("config.toml"));
        let mut config = AtlaConfig::default();
        config.upsert_profile(
            "work",
            Profile {
                instance: "https://work.atlassian.net".to_owned(),
                email: "work@example.com".to_owned(),
                credential_store: CredentialStorage::Keyring,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );
        config.upsert_profile(
            "personal",
            Profile {
                instance: "https://personal.atlassian.net".to_owned(),
                email: "personal@example.com".to_owned(),
                credential_store: CredentialStorage::File,
                default_project: None,
                default_space: None,
                cloud_id: None,
                policy: ProfilePolicy::default(),
            },
        );

        store.save(&config).expect("save config");
        let loaded = store.load().expect("load config");

        assert_eq!(
            loaded.profiles["work"].instance,
            "https://work.atlassian.net"
        );
        assert_eq!(loaded.profiles["work"].email, "work@example.com");
        assert_eq!(
            loaded.profiles["personal"].instance,
            "https://personal.atlassian.net"
        );
        assert_eq!(loaded.profiles["personal"].email, "personal@example.com");
    }
}
