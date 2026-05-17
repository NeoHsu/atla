use anyhow::Context;
use atla_core::auth::{CredentialStore, KeyringCredentialStore};
use atla_core::{
    AtlaConfig, AtlassianClient, ConfigStore, JiraClient, JiraProject, JiraProjectSearch, Profile,
};

use crate::cli::{
    GlobalArgs, JiraCommand, JiraResource, OutputFormat, ProjectAction, ProjectCommand,
};
use crate::config;
use crate::output;

pub async fn run(command: JiraCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        JiraResource::Issue => println!("jira issue commands are planned"),
        JiraResource::Project(command) => run_project(command, global).await?,
        JiraResource::Sprint => println!("jira sprint commands are planned"),
        JiraResource::Board => println!("jira board commands are planned"),
        JiraResource::Search { jql } => println!("jira search is planned: {jql}"),
    }

    Ok(())
}

async fn run_project(command: ProjectCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        ProjectAction::List { query, limit } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
            let search = JiraProjectSearch {
                start_at: 0,
                max_results: limit.clamp(1, 100),
                query,
            };

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/project/search?startAt={}&maxResults={}",
                    profile.instance.trim_end_matches('/'),
                    search.start_at,
                    search.max_results
                );
                if let Some(query) = &search.query {
                    println!("Would GET {url} with query `{query}` using profile `{profile_name}`");
                } else {
                    println!("Would GET {url} using profile `{profile_name}`");
                }
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
            let page = client.search_projects(&search).await.with_context(|| {
                format!(
                    "failed to list Jira projects from {}",
                    client.instance_url()
                )
            })?;

            print_projects(&page.values, page.total, global)?;
        }
        ProjectAction::View { key } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                let url = format!(
                    "{}/rest/api/3/project/{}",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = JiraClient::new(AtlassianClient::from_profile(profile, token));
            let project = client.get_project(&key).await.with_context(|| {
                format!(
                    "failed to load Jira project `{key}` from {}",
                    client.instance_url()
                )
            })?;

            print_project(&project, global)?;
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

fn token_for_profile(profile_name: &str, profile: &Profile) -> anyhow::Result<String> {
    let credential = profile.credential_ref(profile_name);
    let token = KeyringCredentialStore::default()
        .get_token(&credential)
        .context("failed to read API token from keyring")?;

    token.ok_or_else(|| {
        anyhow::anyhow!("missing API token; run `atla auth login --profile {profile_name}`")
    })
}

fn print_projects(
    projects: &[JiraProject],
    total: Option<u64>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(projects),
        OutputFormat::Keys => {
            for project in projects {
                if let Some(key) = &project.key {
                    println!("{key}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,name,type,style,archived");
            for project in projects {
                println!(
                    "{},{},{},{},{}",
                    csv_cell(project.key.as_deref().unwrap_or_default()),
                    csv_cell(project.name.as_deref().unwrap_or_default()),
                    csv_cell(project.project_type_key.as_deref().unwrap_or_default()),
                    csv_cell(project.style.as_deref().unwrap_or_default()),
                    csv_cell(&project.archived.unwrap_or(false).to_string())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<12} {:<16} {:<12} NAME", "KEY", "TYPE", "STYLE");
            for project in projects {
                println!(
                    "{:<12} {:<16} {:<12} {}",
                    project.key.as_deref().unwrap_or("-"),
                    project.project_type_key.as_deref().unwrap_or("-"),
                    project.style.as_deref().unwrap_or("-"),
                    project.name.as_deref().unwrap_or("-")
                );
            }

            if let Some(total) = total {
                println!();
                println!("Showing {} of {total} projects.", projects.len());
            }
            Ok(())
        }
    }
}

fn print_project(project: &JiraProject, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(project),
        OutputFormat::Keys => {
            if let Some(key) = &project.key {
                println!("{key}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,name,type,style,archived,id");
            println!(
                "{},{},{},{},{},{}",
                csv_cell(project.key.as_deref().unwrap_or_default()),
                csv_cell(project.name.as_deref().unwrap_or_default()),
                csv_cell(project.project_type_key.as_deref().unwrap_or_default()),
                csv_cell(project.style.as_deref().unwrap_or_default()),
                csv_cell(&project.archived.unwrap_or(false).to_string()),
                csv_cell(project.id.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("Key: {}", project.key.as_deref().unwrap_or("-"));
            println!("Name: {}", project.name.as_deref().unwrap_or("-"));
            println!(
                "Type: {}",
                project.project_type_key.as_deref().unwrap_or("-")
            );
            println!("Style: {}", project.style.as_deref().unwrap_or("-"));
            println!("Archived: {}", project.archived.unwrap_or(false));
            if let Some(id) = &project.id {
                println!("ID: {id}");
            }
            Ok(())
        }
    }
}

fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}
