use std::time::Duration;

use anyhow::Context;
use atla_core::auth::{CredentialStore, env_token};
use atla_core::{
    AtlassianClient, AtlassianProduct, ConfigStore, ConfluenceClient, CredentialStorage,
    FileCredentialStore, HttpPolicy, JiraClient, KeyringCredentialStore, Profile,
};

use crate::cli::GlobalArgs;
use crate::config;
use crate::error::AuthSetupError;

#[derive(Debug, Clone)]
pub struct AppContext {
    profile_name: String,
    profile: Profile,
    http_policy: HttpPolicy,
    max_pages: Option<u32>,
    max_items: Option<u32>,
    verbose: bool,
}

impl AppContext {
    pub fn load(global: &GlobalArgs) -> anyhow::Result<Self> {
        let store = ConfigStore::default_store().context("failed to find config location")?;
        let atla_config = if global.read_only {
            store.load_read_only()
        } else {
            store.load()
        }
        .context("failed to load config")?;
        let active_profile_name = config::active_profile(global);
        let (profile_name, profile) = atla_config
            .active_profile(active_profile_name)
            .ok_or_else(|| {
                let message = if let Some(name) = active_profile_name {
                    format!(
                        "profile `{name}` does not exist; run `atla auth login --profile {name}` to create it"
                    )
                } else if let Some(name) = atla_config.default.profile.as_deref() {
                    format!(
                        "default profile `{name}` does not exist; run `atla auth login --profile {name}` to recreate it"
                    )
                } else {
                    "no active profile; run `atla auth login` first".to_owned()
                };
                anyhow::Error::new(AuthSetupError(message))
            })?;

        let http_policy = global.timeout.map_or_else(HttpPolicy::default, |seconds| {
            HttpPolicy::default().with_timeout(Duration::from_secs(seconds))
        });
        crate::output::configure_profile(profile_name);
        Ok(Self {
            profile_name: profile_name.to_owned(),
            profile: profile.clone(),
            http_policy,
            max_pages: global.max_pages,
            max_items: global.max_items,
            verbose: global.verbose,
        })
    }

    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn token(&self) -> anyhow::Result<String> {
        if let Some(token) = env_token() {
            return Ok(token);
        }

        let credential = self.profile.credential_ref(&self.profile_name);
        let token = match self.profile.credential_store {
            CredentialStorage::Keyring => KeyringCredentialStore::default()
                .get_token(&credential)
                .context("failed to read API token from keyring")?,
            CredentialStorage::File => FileCredentialStore::default_store()
                .context("failed to open file credential store")?
                .get_token(&credential)
                .context("failed to read API token from file credential store")?,
        };

        token.ok_or_else(|| {
            anyhow::Error::new(AuthSetupError(format!(
                "missing API token; run `atla auth login --profile {} --storage {}` or set ATLA_TOKEN",
                self.profile_name, self.profile.credential_store
            )))
        })
    }

    pub(crate) fn atlassian_client(
        &self,
        product: AtlassianProduct,
    ) -> anyhow::Result<AtlassianClient> {
        Ok(AtlassianClient::from_profile_for_product_with_policy(
            &self.profile,
            self.token()?,
            product,
            self.http_policy,
        )
        .with_execution_limits(self.max_pages, self.max_items)
        .with_verbose(self.verbose))
    }

    pub fn jira_client(&self) -> anyhow::Result<JiraClient> {
        Ok(JiraClient::new(
            self.atlassian_client(AtlassianProduct::Jira)?,
        ))
    }

    pub fn confluence_client(&self) -> anyhow::Result<ConfluenceClient> {
        Ok(ConfluenceClient::new(
            self.atlassian_client(AtlassianProduct::Confluence)?,
        ))
    }
}
