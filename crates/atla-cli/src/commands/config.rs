use anyhow::{Context, bail};
use atla_core::ConfigStore;

use crate::cli::{ConfigAction, ConfigCommand, GlobalArgs};
use crate::config;
use crate::output;

pub async fn run(command: ConfigCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let mut atla_config = if global.read_only {
        store.load_read_only()
    } else {
        store.load()
    }
    .context("failed to load config")?;

    match command.action {
        ConfigAction::Set { key, value } => {
            atla_config
                .set_value(&key, value.clone(), config::active_profile(global))
                .with_context(|| format!("failed to set config key `{key}`"))?;

            if global.dry_run {
                println!("Would set {key}={value}");
                return Ok(());
            }

            store.save(&atla_config).context("failed to save config")?;
            println!("Set {key}={value}");
        }
        ConfigAction::Get { key } => {
            let Some(value) = atla_config
                .get_value(&key, config::active_profile(global))
                .with_context(|| format!("failed to get config key `{key}`"))?
            else {
                bail!("config key `{key}` is not set");
            };

            println!("{value}");
        }
        ConfigAction::List => {
            let Some(format) = global.output else {
                print_config(&atla_config);
                return Ok(());
            };

            let entries = config_entries(&atla_config);
            let keys = entries.iter().map(|(key, _)| key.clone()).collect();
            let rows = entries
                .into_iter()
                .map(|(key, value)| vec![key, value])
                .collect();

            output::print_records(format, &atla_config, keys, &["key", "value"], rows, None)?;
        }
    }

    Ok(())
}

fn print_config(config: &atla_core::AtlaConfig) {
    let default = config.default.profile.as_deref().unwrap_or("<none>");
    println!("schema_version = {}", config.schema_version);
    println!("default.profile = {default}");

    for (name, profile) in &config.profiles {
        println!();
        println!("[profiles.{name}]");
        println!("instance = {}", profile.instance);
        println!("email = {}", profile.email);
        println!("credential_store = {}", profile.credential_store);
        if let Some(cloud_id) = &profile.cloud_id {
            println!("cloud_id = {cloud_id}");
        }

        if let Some(default_project) = &profile.default_project {
            println!("default_project = {default_project}");
        }

        if let Some(default_space) = &profile.default_space {
            println!("default_space = {default_space}");
        }
        println!("policy.mode = {}", profile.policy.mode);
        if !profile.policy.allow.is_empty() {
            println!("policy.allow = {}", profile.policy.allow.join(","));
        }
        if !profile.policy.deny.is_empty() {
            println!("policy.deny = {}", profile.policy.deny.join(","));
        }
    }

    if !config.aliases.is_empty() {
        println!();
        println!("[aliases]");
        for (name, expansion) in &config.aliases {
            println!("{name} = {expansion}");
        }
    }
}

fn config_entries(config: &atla_core::AtlaConfig) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    entries.push((
        "schema_version".to_owned(),
        config.schema_version.to_string(),
    ));
    entries.push((
        "default.profile".to_owned(),
        config
            .default
            .profile
            .clone()
            .unwrap_or_else(|| "<none>".to_owned()),
    ));

    for (name, profile) in &config.profiles {
        let prefix = format!("profiles.{name}");
        entries.push((format!("{prefix}.instance"), profile.instance.clone()));
        entries.push((format!("{prefix}.email"), profile.email.clone()));
        entries.push((
            format!("{prefix}.credential_store"),
            profile.credential_store.to_string(),
        ));
        if let Some(cloud_id) = &profile.cloud_id {
            entries.push((format!("{prefix}.cloud_id"), cloud_id.clone()));
        }

        if let Some(default_project) = &profile.default_project {
            entries.push((format!("{prefix}.default_project"), default_project.clone()));
        }

        if let Some(default_space) = &profile.default_space {
            entries.push((format!("{prefix}.default_space"), default_space.clone()));
        }
        entries.push((
            format!("{prefix}.policy.mode"),
            profile.policy.mode.to_string(),
        ));
        if !profile.policy.allow.is_empty() {
            entries.push((
                format!("{prefix}.policy.allow"),
                profile.policy.allow.join(","),
            ));
        }
        if !profile.policy.deny.is_empty() {
            entries.push((
                format!("{prefix}.policy.deny"),
                profile.policy.deny.join(","),
            ));
        }
    }

    for (name, expansion) in &config.aliases {
        entries.push((format!("aliases.{name}"), expansion.clone()));
    }

    entries
}
