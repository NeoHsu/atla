use anyhow::{Context, bail};
use atla_core::ConfigStore;

use crate::cli::{ConfigAction, ConfigCommand, GlobalArgs};
use crate::config;
use crate::output;

pub async fn run(command: ConfigCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let mut atla_config = store.load().context("failed to load config")?;

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
            if matches!(global.output, Some(crate::cli::OutputFormat::Json)) {
                output::print_json(&atla_config)?;
            } else {
                print_config(&atla_config);
            }
        }
    }

    Ok(())
}

fn print_config(config: &atla_core::AtlaConfig) {
    let default = config.default.profile.as_deref().unwrap_or("<none>");
    println!("default.profile = {default}");

    for (name, profile) in &config.profiles {
        println!();
        println!("[profiles.{name}]");
        println!("instance = {}", profile.instance);
        println!("email = {}", profile.email);

        if let Some(default_project) = &profile.default_project {
            println!("default_project = {default_project}");
        }

        if let Some(default_space) = &profile.default_space {
            println!("default_space = {default_space}");
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
