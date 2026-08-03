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

mod config;
mod policy;
mod store;

pub use config::{AtlaConfig, AtlassianProduct, DefaultSection, Profile};
pub use policy::{PolicyDecisionSource, PolicyMode, ProfilePolicy, ProfilePolicyDecision};
pub use store::ConfigStore;

#[cfg(test)]
mod tests;
