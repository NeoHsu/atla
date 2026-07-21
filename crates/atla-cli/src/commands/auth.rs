use std::io::{IsTerminal, Read, stdin, stdout};
use std::time::Duration;

use anyhow::{Context, bail};
use atla_core::auth::{CredentialStore, env_token};
use atla_core::profile::normalize_cloud_id;
use atla_core::{
    AtlaConfig, AtlassianInstance, ConfigStore, CredentialStorage, FileCredentialStore, HttpPolicy,
    KeyringCredentialStore, Profile, ProfilePolicy, discover_tenant,
};
use dialoguer::{Input, Password};

use crate::cli::{AuthAction, AuthCommand, AuthStorage, GlobalArgs, OutputFormat};
use crate::config;
use crate::output;

#[derive(serde::Serialize)]
struct AuthStatusOutput<'a> {
    configured: bool,
    profile: &'a str,
    instance: &'a str,
    email: &'a str,
    credential_store: String,
    api_target: &'static str,
    cloud_id: Option<&'a str>,
    policy_mode: String,
    token: String,
}

pub async fn run(command: AuthCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let mut atla_config = if global.read_only {
        store.load_read_only()
    } else {
        store.load()
    }
    .context("failed to load config")?;
    match command.action {
        AuthAction::Login {
            instance,
            cloud_id,
            email,
            token,
            token_stdin,
            storage,
        } => {
            let profile_name = config::active_profile(global).unwrap_or("default");
            let instance = normalize_instance(&required_text(
                "Atlassian instance URL",
                "--instance",
                instance,
                global,
            )?);
            let cloud_id = cloud_id
                .as_deref()
                .map(normalize_cloud_id)
                .transpose()?
                .flatten();
            let email = required_text("Email", "--email", email, global)?;
            let token = if token_stdin {
                read_token_stdin()?
            } else {
                required_secret("API token", "--token or --token-stdin", token, global)?
            };
            let credential_store = storage
                .map(Into::into)
                .or_else(|| {
                    atla_config
                        .profiles
                        .get(profile_name)
                        .map(|profile| profile.credential_store)
                })
                .unwrap_or(CredentialStorage::Keyring);

            let profile = Profile {
                instance,
                email,
                credential_store,
                default_project: None,
                default_space: None,
                cloud_id,
                policy: ProfilePolicy::default(),
            };
            let credential = profile.credential_ref(profile_name);

            if global.dry_run {
                println!("Would save credentials for profile `{profile_name}`");
                return Ok(());
            }

            save_token(credential_store, &credential, &token)
                .with_context(|| format!("failed to save API token to {credential_store}"))?;
            atla_config.upsert_profile(profile_name, profile);
            store.save(&atla_config).context("failed to save config")?;

            println!(
                "Logged in to {} as {} using profile `{}` ({credential_store})",
                credential.instance, credential.email, credential.profile
            );
            if matches!(credential_store, CredentialStorage::File) {
                println!(
                    "Stored token in {}. Keep this file private.",
                    FileCredentialStore::default_path()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|_| "credentials.toml".to_owned())
                );
            }
        }
        AuthAction::Discover { site } => {
            let site = normalize_instance(&site);
            let url = format!("{}/_edge/tenant_info", site.trim_end_matches('/'));
            if global.dry_run {
                println!("Would GET {url}");
                return Ok(());
            }
            let policy = global.timeout.map_or_else(HttpPolicy::default, |seconds| {
                HttpPolicy::default().with_timeout(Duration::from_secs(seconds))
            });
            let discovery = discover_tenant(&site, policy)
                .await
                .with_context(|| format!("failed to discover Atlassian tenant at {site}"))?;
            match global.output {
                Some(OutputFormat::Json) => output::print_json(&discovery)?,
                Some(format @ (OutputFormat::Table | OutputFormat::Csv | OutputFormat::Keys)) => {
                    output::print_records(
                        format,
                        &discovery,
                        vec![discovery.cloud_id.clone()],
                        &["site", "cloud_id", "jira_endpoint", "confluence_endpoint"],
                        vec![vec![
                            discovery.site.clone(),
                            discovery.cloud_id.clone(),
                            discovery.jira_endpoint.clone(),
                            discovery.confluence_endpoint.clone(),
                        ]],
                        None,
                    )?;
                }
                None => {
                    println!("Site: {}", discovery.site);
                    println!("Cloud ID: {}", discovery.cloud_id);
                    println!("Jira endpoint: {}", discovery.jira_endpoint);
                    println!("Confluence endpoint: {}", discovery.confluence_endpoint);
                }
            }
        }
        AuthAction::Logout { .. } => {
            let (profile_name, profile) = active_profile(&atla_config, global)?;
            let credential = profile.credential_ref(profile_name);

            if global.dry_run {
                println!("Would remove credentials for profile `{profile_name}`");
                return Ok(());
            }

            delete_token(profile.credential_store, &credential).with_context(|| {
                format!(
                    "failed to remove API token from {}",
                    profile.credential_store
                )
            })?;
            println!(
                "Logged out profile `{profile_name}` ({})",
                profile.credential_store
            );
        }
        AuthAction::Status => {
            let Some((profile_name, profile)) =
                atla_config.active_profile(config::active_profile(global))
            else {
                let token = if env_token().is_some() {
                    "provided by environment"
                } else {
                    "missing"
                };
                let status = serde_json::json!({
                    "configured": false,
                    "profile": serde_json::Value::Null,
                    "instance": serde_json::Value::Null,
                    "email": serde_json::Value::Null,
                    "credential_store": serde_json::Value::Null,
                    "api_target": serde_json::Value::Null,
                    "cloud_id": serde_json::Value::Null,
                    "policy_mode": serde_json::Value::Null,
                    "token": token,
                });
                match global.output {
                    Some(OutputFormat::Json) => output::print_json(&status)?,
                    Some(
                        format @ (OutputFormat::Table | OutputFormat::Csv | OutputFormat::Keys),
                    ) => {
                        output::print_records(
                            format,
                            &status,
                            Vec::new(),
                            &["configured", "profile", "instance", "api_target", "token"],
                            vec![vec![
                                "false".to_owned(),
                                "<none>".to_owned(),
                                "<none>".to_owned(),
                                "<none>".to_owned(),
                                token.to_owned(),
                            ]],
                            None,
                        )?;
                    }
                    None => println!("Not logged in"),
                }
                return Ok(());
            };
            let credential = profile.credential_ref(profile_name);
            let token_status = token_status(profile.credential_store, &credential);
            let status = AuthStatusOutput {
                configured: true,
                profile: profile_name,
                instance: &profile.instance,
                email: &profile.email,
                credential_store: profile.credential_store.to_string(),
                api_target: if profile.uses_scoped_token_gateway() {
                    "scoped-token-gateway"
                } else {
                    "site"
                },
                cloud_id: profile.cloud_id.as_deref(),
                policy_mode: profile.policy.mode.to_string(),
                token: token_status,
            };

            match global.output {
                Some(OutputFormat::Json) => output::print_json(&status)?,
                Some(format @ (OutputFormat::Table | OutputFormat::Csv | OutputFormat::Keys)) => {
                    output::print_records(
                        format,
                        &status,
                        vec![status.profile.to_owned()],
                        &[
                            "configured",
                            "profile",
                            "instance",
                            "email",
                            "credential_store",
                            "api_target",
                            "cloud_id",
                            "policy_mode",
                            "token",
                        ],
                        vec![vec![
                            status.configured.to_string(),
                            status.profile.to_owned(),
                            status.instance.to_owned(),
                            status.email.to_owned(),
                            status.credential_store.clone(),
                            status.api_target.to_owned(),
                            status.cloud_id.unwrap_or("").to_owned(),
                            status.policy_mode.clone(),
                            status.token.clone(),
                        ]],
                        None,
                    )?
                }
                None => {
                    println!("Configured: {}", status.configured);
                    println!("Profile: {}", status.profile);
                    println!("Instance: {}", status.instance);
                    println!("Email: {}", status.email);
                    println!("Credential store: {}", status.credential_store);
                    println!("API target: {}", status.api_target);
                    if let Some(cloud_id) = status.cloud_id {
                        println!("Cloud ID: {cloud_id}");
                    }
                    println!("Policy mode: {}", status.policy_mode);
                    println!("Token: {}", status.token);
                }
            }
        }
        AuthAction::Switch { profile } => {
            atla_config
                .switch_profile(&profile)
                .with_context(|| format!("failed to switch to profile `{profile}`"))?;

            if global.dry_run {
                println!("Would switch default profile to `{profile}`");
                return Ok(());
            }

            store.save(&atla_config).context("failed to save config")?;
            println!("Switched default profile to `{profile}`");
        }
    }

    Ok(())
}

impl From<AuthStorage> for CredentialStorage {
    fn from(storage: AuthStorage) -> Self {
        match storage {
            AuthStorage::Keyring => Self::Keyring,
            AuthStorage::File => Self::File,
        }
    }
}

fn save_token(
    storage: CredentialStorage,
    credential: &atla_core::CredentialRef,
    token: &str,
) -> anyhow::Result<()> {
    match storage {
        CredentialStorage::Keyring => {
            Ok(KeyringCredentialStore::default().save_token(credential, token)?)
        }
        CredentialStorage::File => {
            Ok(FileCredentialStore::default_store()?.save_token(credential, token)?)
        }
    }
}

fn delete_token(
    storage: CredentialStorage,
    credential: &atla_core::CredentialRef,
) -> anyhow::Result<()> {
    match storage {
        CredentialStorage::Keyring => {
            Ok(KeyringCredentialStore::default().delete_token(credential)?)
        }
        CredentialStorage::File => {
            Ok(FileCredentialStore::default_store()?.delete_token(credential)?)
        }
    }
}

fn token_status(storage: CredentialStorage, credential: &atla_core::CredentialRef) -> String {
    if env_token().is_some() {
        return "provided by environment".to_owned();
    }

    let result = match storage {
        CredentialStorage::Keyring => KeyringCredentialStore::default().has_token(credential),
        CredentialStorage::File => {
            FileCredentialStore::default_store().and_then(|store| store.has_token(credential))
        }
    };

    match result {
        Ok(true) => format!("stored in {storage}"),
        Ok(false) => "missing".to_owned(),
        Err(error) => format!("{storage} unavailable: {error}"),
    }
}

fn active_profile<'a>(
    atla_config: &'a AtlaConfig,
    global: &GlobalArgs,
) -> anyhow::Result<(&'a str, &'a Profile)> {
    atla_config
        .active_profile(config::active_profile(global))
        .ok_or_else(|| anyhow::anyhow!("no active profile; run `atla auth login` first"))
}

fn required_text(
    prompt: &str,
    flag: &str,
    value: Option<String>,
    global: &GlobalArgs,
) -> anyhow::Result<String> {
    if let Some(value) = value {
        return Ok(value);
    }

    if can_prompt(global) {
        return Input::new()
            .with_prompt(prompt)
            .interact_text()
            .context("failed to read prompt input");
    }

    bail!("missing required flag: {flag}");
}

fn read_token_stdin() -> anyhow::Result<String> {
    const MAX_TOKEN_BYTES: u64 = 64 * 1024;

    let mut input = String::new();
    stdin()
        .take(MAX_TOKEN_BYTES + 1)
        .read_to_string(&mut input)
        .context("failed to read API token from stdin")?;
    if input.len() as u64 > MAX_TOKEN_BYTES {
        bail!("API token from stdin exceeds {MAX_TOKEN_BYTES} bytes");
    }
    let token = input.trim_end_matches(['\r', '\n']);
    if token.is_empty() {
        bail!("API token from stdin is empty");
    }
    if token.contains(['\r', '\n']) {
        bail!("API token from stdin must be a single line");
    }
    Ok(token.to_owned())
}

fn required_secret(
    prompt: &str,
    flag: &str,
    value: Option<String>,
    global: &GlobalArgs,
) -> anyhow::Result<String> {
    if let Some(value) = value {
        return Ok(value);
    }

    if can_prompt(global) {
        return Password::new()
            .with_prompt(prompt)
            .interact()
            .context("failed to read prompt input");
    }

    bail!("missing required flag: {flag}");
}

fn can_prompt(global: &GlobalArgs) -> bool {
    !global.no_input && stdin().is_terminal() && stdout().is_terminal()
}

fn normalize_instance(instance: &str) -> String {
    let instance = instance.trim().trim_end_matches('/');
    let instance = if instance.starts_with("http://") || instance.starts_with("https://") {
        instance.to_owned()
    } else {
        format!("https://{instance}")
    };

    AtlassianInstance::new(instance).base_url
}
