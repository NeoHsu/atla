use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::{CredentialRef, CredentialStorage};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlaConfig {
    #[serde(default)]
    pub default: DefaultSection,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub instance: String,
    pub email: String,
    #[serde(default)]
    pub credential_store: CredentialStorage,
    pub default_project: Option<String>,
    pub default_space: Option<String>,
}

impl Profile {
    pub fn credential_ref(&self, name: impl Into<String>) -> CredentialRef {
        CredentialRef {
            profile: name.into(),
            email: self.email.clone(),
            instance: self.instance.clone(),
        }
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

        let base_dirs = BaseDirs::new().ok_or(ProfileError::ConfigDirUnavailable)?;
        Ok(base_dirs.config_dir().join("atla").join("config.toml"))
    }

    pub fn default_store() -> Result<Self, ProfileError> {
        Ok(Self::new(Self::default_path()?))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<AtlaConfig, ProfileError> {
        if !self.path.exists() {
            return Ok(AtlaConfig::default());
        }

        let contents = fs::read_to_string(&self.path)?;
        toml::from_str(&contents).map_err(|error| ProfileError::Decode(error.to_string()))
    }

    pub fn save(&self, config: &AtlaConfig) -> Result<(), ProfileError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(config)
            .map_err(|error| ProfileError::Encode(error.to_string()))?;
        fs::write(&self.path, contents)?;
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
    #[error("could not parse config: {0}")]
    Decode(String),
    #[error("could not encode config: {0}")]
    Encode(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
            },
        );

        store.save(&config).expect("save config");
        let loaded = store.load().expect("load config");

        assert_eq!(loaded.default.profile.as_deref(), Some("work"));
        assert_eq!(
            loaded.profiles["work"].default_project.as_deref(),
            Some("PROJ")
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
            },
        );

        config.switch_profile("work").expect("switch profile");
        assert!(matches!(
            config.switch_profile("missing"),
            Err(ProfileError::MissingProfile(name)) if name == "missing"
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
}
