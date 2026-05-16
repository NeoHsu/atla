use std::io::{IsTerminal, stdin, stdout};

use anyhow::{Context, bail};
use atla_core::auth::{CredentialStore, KeyringCredentialStore};
use atla_core::{AtlaConfig, AtlassianInstance, ConfigStore, Profile};
use dialoguer::{Input, Password};

use crate::cli::{AuthAction, AuthCommand, GlobalArgs};
use crate::config;

pub async fn run(command: AuthCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let mut atla_config = store.load().context("failed to load config")?;
    let credential_store = KeyringCredentialStore::default();

    match command.action {
        AuthAction::Login {
            instance,
            email,
            token,
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

            let profile = Profile {
                instance,
                email,
                default_project: None,
                default_space: None,
            };
            let credential = profile.credential_ref(profile_name);

            if global.dry_run {
                println!("Would save credentials for profile `{profile_name}`");
                return Ok(());
            }

            credential_store
                .save_token(&credential, &token)
                .context("failed to save API token to keyring")?;
            atla_config.upsert_profile(profile_name, profile);
            store.save(&atla_config).context("failed to save config")?;

            println!(
                "Logged in to {} as {} using profile `{}`",
                credential.instance, credential.email, credential.profile
            );
        }
        AuthAction::Logout => {
            let (profile_name, profile) = active_profile(&atla_config, global)?;
            let credential = profile.credential_ref(profile_name);

            if global.dry_run {
                println!("Would remove credentials for profile `{profile_name}`");
                return Ok(());
            }

            credential_store
                .delete_token(&credential)
                .context("failed to remove API token from keyring")?;
            println!("Logged out profile `{profile_name}`");
        }
        AuthAction::Status => {
            let Some((profile_name, profile)) =
                atla_config.active_profile(config::active_profile(global))
            else {
                println!("Not logged in");
                return Ok(());
            };
            let credential = profile.credential_ref(profile_name);
            let has_token = credential_store
                .has_token(&credential)
                .context("failed to read API token from keyring")?;

            println!("Profile: {profile_name}");
            println!("Instance: {}", profile.instance);
            println!("Email: {}", profile.email);
            println!("Token: {}", if has_token { "stored" } else { "missing" });
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
