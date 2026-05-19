use std::io::{IsTerminal, stdin, stdout};

use anyhow::{Context, bail};
use atla_core::auth::{CredentialStore, env_token};
use atla_core::{
    AtlaConfig, AtlassianInstance, ConfigStore, CredentialStorage, FileCredentialStore,
    KeyringCredentialStore, Profile,
};
use dialoguer::{Input, Password};

use crate::cli::{AuthAction, AuthCommand, AuthStorage, GlobalArgs};
use crate::config;

pub async fn run(command: AuthCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let mut atla_config = store.load().context("failed to load config")?;
    match command.action {
        AuthAction::Login {
            instance,
            email,
            token,
            storage,
        } => {
            let profile_name = config::active_profile(global).unwrap_or("default");
            let instance = normalize_instance(&required_text(
                "Atlassian instance URL",
                "--instance",
                instance,
                global,
            )?);
            let email = required_text("Email", "--email", email, global)?;
            let token = required_secret("API token", "--token", token, global)?;
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
        AuthAction::Logout => {
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
                println!("Not logged in");
                return Ok(());
            };
            let credential = profile.credential_ref(profile_name);
            let token_status = token_status(profile.credential_store, &credential);

            println!("Profile: {profile_name}");
            println!("Instance: {}", profile.instance);
            println!("Email: {}", profile.email);
            println!("Credential store: {}", profile.credential_store);
            println!("Token: {token_status}");
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
