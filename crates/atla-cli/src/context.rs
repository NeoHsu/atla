use anyhow::Context;
use atla_core::auth::{CredentialStore, KeyringCredentialStore};
use atla_core::{AtlassianClient, ConfigStore, ConfluenceClient, JiraClient, Profile};

use crate::cli::GlobalArgs;
use crate::config;

#[derive(Debug, Clone)]
pub struct AppContext {
    profile_name: String,
    profile: Profile,
}

impl AppContext {
    pub fn load(global: &GlobalArgs) -> anyhow::Result<Self> {
        let store = ConfigStore::default_store().context("failed to find config location")?;
        let atla_config = store.load().context("failed to load config")?;
        let (profile_name, profile) = atla_config
            .active_profile(config::active_profile(global))
            .ok_or_else(|| anyhow::anyhow!("no active profile; run `atla auth login` first"))?;

        Ok(Self {
            profile_name: profile_name.to_owned(),
            profile: profile.clone(),
        })
    }

    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn token(&self) -> anyhow::Result<String> {
        let credential = self.profile.credential_ref(&self.profile_name);
        let token = KeyringCredentialStore::default()
            .get_token(&credential)
            .context("failed to read API token from keyring")?;

        token.ok_or_else(|| {
            anyhow::anyhow!(
                "missing API token; run `atla auth login --profile {}`",
                self.profile_name
            )
        })
    }

    pub fn atlassian_client(&self) -> anyhow::Result<AtlassianClient> {
        Ok(AtlassianClient::from_profile(&self.profile, self.token()?))
    }

    pub fn jira_client(&self) -> anyhow::Result<JiraClient> {
        Ok(JiraClient::new(self.atlassian_client()?))
    }

    pub fn confluence_client(&self) -> anyhow::Result<ConfluenceClient> {
        Ok(ConfluenceClient::new(self.atlassian_client()?))
    }
}
