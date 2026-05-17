use anyhow::Context;
use atla_core::auth::{CredentialStore, KeyringCredentialStore};
use atla_core::{
    AtlaConfig, AtlassianClient, ConfigStore, ConfluenceClient, ConfluencePage,
    ConfluencePageSearch, ConfluenceSpace, ConfluenceSpaceSearch, Profile,
};

use crate::cli::{
    ConfluenceCommand, ConfluenceResource, GlobalArgs, OutputFormat, PageAction, PageCommand,
    SpaceAction, SpaceCommand,
};
use crate::config;
use crate::output;

pub async fn run(command: ConfluenceCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        ConfluenceResource::Page(command) => run_page(command, global).await?,
        ConfluenceResource::Space(command) => run_space(command, global).await?,
        ConfluenceResource::Blog => println!("confluence blog commands are planned"),
        ConfluenceResource::Search { cql } => println!("confluence search is planned: {cql}"),
        ConfluenceResource::Attachment => println!("confluence attachment commands are planned"),
    }

    Ok(())
}

async fn run_page(command: PageCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        PageAction::List {
            space,
            space_id,
            title,
            limit,
        } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
            let limit = limit.clamp(1, 250);

            if global.dry_run {
                if let Some(space) = &space {
                    let space_url = format!(
                        "{}/wiki/api/v2/spaces?keys={space}&limit=1",
                        profile.instance.trim_end_matches('/')
                    );
                    println!("Would GET {space_url} using profile `{profile_name}`");
                }

                let mut url = format!(
                    "{}/wiki/api/v2/pages?limit={limit}",
                    profile.instance.trim_end_matches('/')
                );
                if let Some(space_id) = &space_id {
                    url.push_str(&format!("&space-id={space_id}"));
                } else if space.is_some() {
                    url.push_str("&space-id=<resolved-space-id>");
                }
                if let Some(title) = &title {
                    url.push_str(&format!("&title={title}"));
                }
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let resolved_space_id = resolve_space_id(&client, space.as_deref(), space_id).await?;
            let search = ConfluencePageSearch {
                space_id: resolved_space_id,
                title,
                limit,
            };
            let page = client.list_pages(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence pages from {}",
                    client.instance_url()
                )
            })?;

            print_pages(&page.results, global)?;
        }
        PageAction::View { id } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/pages/{}",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let page = client.get_page(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence page `{id}` from {}",
                    client.instance_url()
                )
            })?;

            print_page(&page, global)?;
        }
    }

    Ok(())
}

async fn run_space(command: SpaceCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        SpaceAction::List { key, limit } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
            let search = ConfluenceSpaceSearch {
                key,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                let mut url = format!(
                    "{}/wiki/api/v2/spaces?limit={}",
                    profile.instance.trim_end_matches('/'),
                    search.limit
                );
                if let Some(key) = &search.key {
                    url.push_str(&format!("&keys={key}"));
                }
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let page = client.list_spaces(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence spaces from {}",
                    client.instance_url()
                )
            })?;

            print_spaces(&page.results, global)?;
        }
        SpaceAction::View { key } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/spaces?keys={}&limit=1",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let space = client.get_space_by_key(&key).await.with_context(|| {
                format!(
                    "failed to load Confluence space `{key}` from {}",
                    client.instance_url()
                )
            })?;
            let space =
                space.ok_or_else(|| anyhow::anyhow!("Confluence space `{key}` was not found"))?;

            print_space(&space, global)?;
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

async fn resolve_space_id(
    client: &ConfluenceClient,
    space: Option<&str>,
    space_id: Option<String>,
) -> anyhow::Result<Option<String>> {
    if let Some(space_id) = space_id {
        return Ok(Some(space_id));
    }

    let Some(space_key) = space else {
        return Ok(None);
    };

    let space = client
        .get_space_by_key(space_key)
        .await
        .with_context(|| format!("failed to resolve Confluence space `{space_key}`"))?;
    let space =
        space.ok_or_else(|| anyhow::anyhow!("Confluence space `{space_key}` was not found"))?;
    let space_id = space
        .id
        .ok_or_else(|| anyhow::anyhow!("Confluence space `{space_key}` did not include an id"))?;

    Ok(Some(space_id))
}

fn print_pages(pages: &[ConfluencePage], global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(pages),
        OutputFormat::Keys => {
            for page in pages {
                if let Some(id) = &page.id {
                    println!("{id}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,title,status,space_id,parent_id,version");
            for page in pages {
                println!(
                    "{},{},{},{},{},{}",
                    csv_cell(page.id.as_deref().unwrap_or_default()),
                    csv_cell(page.title.as_deref().unwrap_or_default()),
                    csv_cell(page.status.as_deref().unwrap_or_default()),
                    csv_cell(page.space_id.as_deref().unwrap_or_default()),
                    csv_cell(page.parent_id.as_deref().unwrap_or_default()),
                    csv_cell(&page_version(page).unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<14} {:<12} {:<10} TITLE", "ID", "STATUS", "VERSION");
            for page in pages {
                println!(
                    "{:<14} {:<12} {:<10} {}",
                    page.id.as_deref().unwrap_or("-"),
                    page.status.as_deref().unwrap_or("-"),
                    page_version(page).as_deref().unwrap_or("-"),
                    page.title.as_deref().unwrap_or("-")
                );
            }
            Ok(())
        }
    }
}

fn print_page(page: &ConfluencePage, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(page),
        OutputFormat::Keys => {
            if let Some(id) = &page.id {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,title,status,space_id,parent_id,version");
            println!(
                "{},{},{},{},{},{}",
                csv_cell(page.id.as_deref().unwrap_or_default()),
                csv_cell(page.title.as_deref().unwrap_or_default()),
                csv_cell(page.status.as_deref().unwrap_or_default()),
                csv_cell(page.space_id.as_deref().unwrap_or_default()),
                csv_cell(page.parent_id.as_deref().unwrap_or_default()),
                csv_cell(&page_version(page).unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("ID: {}", page.id.as_deref().unwrap_or("-"));
            println!("Title: {}", page.title.as_deref().unwrap_or("-"));
            println!("Status: {}", page.status.as_deref().unwrap_or("-"));
            println!("Space ID: {}", page.space_id.as_deref().unwrap_or("-"));
            println!("Parent ID: {}", page.parent_id.as_deref().unwrap_or("-"));
            println!("Version: {}", page_version(page).as_deref().unwrap_or("-"));
            Ok(())
        }
    }
}

fn page_version(page: &ConfluencePage) -> Option<String> {
    page.version
        .as_ref()
        .and_then(|version| version.number)
        .map(|number| number.to_string())
}

fn print_spaces(spaces: &[ConfluenceSpace], global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(spaces),
        OutputFormat::Keys => {
            for space in spaces {
                if let Some(key) = &space.key {
                    println!("{key}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,name,type,status,id,homepage_id");
            for space in spaces {
                println!(
                    "{},{},{},{},{},{}",
                    csv_cell(space.key.as_deref().unwrap_or_default()),
                    csv_cell(space.name.as_deref().unwrap_or_default()),
                    csv_cell(space.space_type.as_deref().unwrap_or_default()),
                    csv_cell(space.status.as_deref().unwrap_or_default()),
                    csv_cell(space.id.as_deref().unwrap_or_default()),
                    csv_cell(space.homepage_id.as_deref().unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<12} {:<16} {:<12} NAME", "KEY", "TYPE", "STATUS");
            for space in spaces {
                println!(
                    "{:<12} {:<16} {:<12} {}",
                    space.key.as_deref().unwrap_or("-"),
                    space.space_type.as_deref().unwrap_or("-"),
                    space.status.as_deref().unwrap_or("-"),
                    space.name.as_deref().unwrap_or("-")
                );
            }
            Ok(())
        }
    }
}

fn print_space(space: &ConfluenceSpace, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(space),
        OutputFormat::Keys => {
            if let Some(key) = &space.key {
                println!("{key}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("key,name,type,status,id,homepage_id");
            println!(
                "{},{},{},{},{},{}",
                csv_cell(space.key.as_deref().unwrap_or_default()),
                csv_cell(space.name.as_deref().unwrap_or_default()),
                csv_cell(space.space_type.as_deref().unwrap_or_default()),
                csv_cell(space.status.as_deref().unwrap_or_default()),
                csv_cell(space.id.as_deref().unwrap_or_default()),
                csv_cell(space.homepage_id.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("Key: {}", space.key.as_deref().unwrap_or("-"));
            println!("Name: {}", space.name.as_deref().unwrap_or("-"));
            println!("Type: {}", space.space_type.as_deref().unwrap_or("-"));
            println!("Status: {}", space.status.as_deref().unwrap_or("-"));
            if let Some(id) = &space.id {
                println!("ID: {id}");
            }
            if let Some(homepage_id) = &space.homepage_id {
                println!("Homepage ID: {homepage_id}");
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
