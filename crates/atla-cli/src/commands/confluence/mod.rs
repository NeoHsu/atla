use anyhow::Context;
use atla_core::auth::{CredentialStore, KeyringCredentialStore};
use atla_core::{
    AtlaConfig, AtlassianClient, ConfigStore, ConfluenceAttachment, ConfluenceAttachmentSearch,
    ConfluenceAttachmentUpload, ConfluenceBlogPost, ConfluenceBlogPostCreate,
    ConfluenceBlogPostSearch, ConfluenceBodyRepresentation, ConfluenceClient,
    ConfluenceContentStatus, ConfluencePage, ConfluencePageCreate, ConfluencePageSearch,
    ConfluencePageUpdate, ConfluenceSearch, ConfluenceSearchResult, ConfluenceSpace,
    ConfluenceSpaceSearch, Profile,
};
use std::fs;
use std::path::Path;

use crate::cli::{
    AttachmentAction, AttachmentCommand, BlogAction, BlogCommand, BodyRepresentation,
    ConfluenceCommand, ConfluenceResource, GlobalArgs, OutputFormat, PageAction, PageCommand,
    SpaceAction, SpaceCommand,
};
use crate::config;
use crate::output;

pub async fn run(command: ConfluenceCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.resource {
        ConfluenceResource::Page(command) => run_page(command, global).await?,
        ConfluenceResource::Space(command) => run_space(command, global).await?,
        ConfluenceResource::Blog(command) => run_blog(command, global).await?,
        ConfluenceResource::Search { cql, limit } => run_search(cql, limit, global).await?,
        ConfluenceResource::Attachment(command) => run_attachment(command, global).await?,
    }

    Ok(())
}

async fn run_search(cql: String, limit: u32, global: &GlobalArgs) -> anyhow::Result<()> {
    let store = ConfigStore::default_store().context("failed to find config location")?;
    let atla_config = store.load().context("failed to load config")?;
    let (profile_name, profile) = active_profile(&atla_config, global)?;
    let search = ConfluenceSearch {
        cql,
        limit: limit.clamp(1, 250),
    };

    if global.dry_run {
        println!(
            "Would GET {}/wiki/rest/api/search?limit={} with CQL `{}` using profile `{profile_name}`",
            profile.instance.trim_end_matches('/'),
            search.limit,
            search.cql
        );
        return Ok(());
    }

    let token = token_for_profile(profile_name, profile)?;
    let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
    let page = client.search(&search).await.with_context(|| {
        format!(
            "failed to search Confluence content from {}",
            client.instance_url()
        )
    })?;

    print_search_results(&page.results, global)
}

async fn run_attachment(command: AttachmentCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        AttachmentAction::List {
            page_id,
            filename,
            limit,
        } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
            let search = ConfluenceAttachmentSearch {
                page_id,
                filename,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}/attachments?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.page_id,
                    search.limit
                );
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let page = client
                .list_page_attachments(&search)
                .await
                .with_context(|| {
                    format!(
                        "failed to list Confluence page attachments from {}",
                        client.instance_url()
                    )
                })?;

            print_attachments(&page.results, global)?;
        }
        AttachmentAction::Upload {
            page_id,
            file,
            comment,
            minor_edit,
        } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;
            let upload = ConfluenceAttachmentUpload {
                page_id,
                file,
                comment,
                minor_edit,
            };

            if global.dry_run {
                println!(
                    "Would PUT {}/wiki/rest/api/content/{}/child/attachment with file `{}` using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    upload.page_id,
                    upload.file.display()
                );
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let page = client
                .upload_page_attachment(&upload)
                .await
                .with_context(|| {
                    format!(
                        "failed to upload Confluence page attachment to {}",
                        client.instance_url()
                    )
                })?;

            print_attachments(&page.results, global)?;
        }
    }

    Ok(())
}

async fn run_blog(command: BlogCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        BlogAction::Create {
            space,
            space_id,
            title,
            body,
            body_file,
            representation,
            draft,
            private,
        } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                if let Some(space) = &space {
                    let space_url = format!(
                        "{}/wiki/api/v2/spaces?keys={space}&limit=1",
                        profile.instance.trim_end_matches('/')
                    );
                    println!("Would GET {space_url} using profile `{profile_name}`");
                }
                println!(
                    "Would POST {}/wiki/api/v2/blogposts using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let body = read_body(body, body_file.as_deref())?;
            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let space_id = resolve_required_space_id(&client, space.as_deref(), space_id).await?;
            let post = client
                .create_blog_post(&ConfluenceBlogPostCreate {
                    space_id,
                    title,
                    body,
                    representation: representation.into(),
                    status: status_from_draft(draft),
                    private: private.then_some(true),
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to create Confluence blog post in {}",
                        client.instance_url()
                    )
                })?;

            print_blog_post(&post, global)?;
        }
        BlogAction::List {
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
                    "{}/wiki/api/v2/blogposts?limit={limit}",
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
            let search = ConfluenceBlogPostSearch {
                space_id: resolved_space_id,
                title,
                limit,
            };
            let page = client.list_blog_posts(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence blog posts from {}",
                    client.instance_url()
                )
            })?;

            print_blog_posts(&page.results, global)?;
        }
        BlogAction::View { id } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/blogposts/{}",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let post = client.get_blog_post(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence blog post `{id}` from {}",
                    client.instance_url()
                )
            })?;

            print_blog_post(&post, global)?;
        }
    }

    Ok(())
}

async fn run_page(command: PageCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        PageAction::Create {
            space,
            space_id,
            title,
            parent,
            body,
            body_file,
            representation,
            draft,
            private,
            root_level,
        } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                if let Some(space) = &space {
                    let space_url = format!(
                        "{}/wiki/api/v2/spaces?keys={space}&limit=1",
                        profile.instance.trim_end_matches('/')
                    );
                    println!("Would GET {space_url} using profile `{profile_name}`");
                }
                println!(
                    "Would POST {}/wiki/api/v2/pages using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let body = read_body(body, body_file.as_deref())?;
            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let space_id = resolve_required_space_id(&client, space.as_deref(), space_id).await?;
            let page = client
                .create_page(&ConfluencePageCreate {
                    space_id,
                    title,
                    parent_id: parent,
                    body,
                    representation: representation.into(),
                    status: status_from_draft(draft),
                    private: private.then_some(true),
                    root_level: root_level.then_some(true),
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to create Confluence page in {}",
                        client.instance_url()
                    )
                })?;

            print_page(&page, global)?;
        }
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
        PageAction::Update {
            id,
            title,
            parent,
            body,
            body_file,
            representation,
            version,
            message,
            draft,
        } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                let endpoint = if body.is_none() && body_file.is_none() && parent.is_none() {
                    format!(
                        "{}/wiki/api/v2/pages/{}/title",
                        profile.instance.trim_end_matches('/'),
                        id
                    )
                } else {
                    format!(
                        "{}/wiki/api/v2/pages/{}",
                        profile.instance.trim_end_matches('/'),
                        id
                    )
                };
                println!("Would PUT {endpoint} using profile `{profile_name}`");
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let existing = client.get_page(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence page `{id}` from {}",
                    client.instance_url()
                )
            })?;
            let status = status_from_draft(draft);

            if body.is_none() && body_file.is_none() && parent.is_none() {
                let title = title.ok_or_else(|| {
                    anyhow::anyhow!(
                        "nothing to update; provide --title, --body-file, --body, or --parent"
                    )
                })?;
                let page = client
                    .update_page_title(&id, &title, status)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to update Confluence page title `{id}` from {}",
                            client.instance_url()
                        )
                    })?;
                print_page(&page, global)?;
                return Ok(());
            }

            let body = read_body(body, body_file.as_deref())?.ok_or_else(|| {
                anyhow::anyhow!("page body update and move require --body or --body-file")
            })?;
            let title = title
                .or(existing.title)
                .ok_or_else(|| anyhow::anyhow!("page `{id}` did not include a title"))?;
            let next_version = version
                .or_else(|| {
                    existing
                        .version
                        .as_ref()
                        .and_then(|version| version.number)
                        .map(|number| number + 1)
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("page `{id}` did not include a version; pass --version")
                })?;

            let page = client
                .update_page(&ConfluencePageUpdate {
                    id: id.clone(),
                    status,
                    title,
                    space_id: existing.space_id,
                    parent_id: parent,
                    body,
                    representation: representation.into(),
                    version: next_version,
                    message,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to update Confluence page `{id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_page(&page, global)?;
        }
        PageAction::Delete {
            id,
            purge,
            draft,
            yes,
        } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/api/v2/pages/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            if !yes {
                anyhow::bail!("refusing to delete page `{id}` without --yes");
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            client
                .delete_page(&id, purge, draft)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete Confluence page `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            println!("Deleted Confluence page {id}");
        }
        PageAction::Move { id, parent } => {
            let store = ConfigStore::default_store().context("failed to find config location")?;
            let atla_config = store.load().context("failed to load config")?;
            let (profile_name, profile) = active_profile(&atla_config, global)?;

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}?body-format=storage using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                println!(
                    "Would PUT {}/wiki/api/v2/pages/{} with parentId `{}` using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id,
                    parent
                );
                return Ok(());
            }

            let token = token_for_profile(profile_name, profile)?;
            let client = ConfluenceClient::new(AtlassianClient::from_profile(profile, token));
            let page = client.move_page(&id, &parent).await.with_context(|| {
                format!(
                    "failed to move Confluence page `{id}` under `{parent}` from {}",
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

async fn resolve_required_space_id(
    client: &ConfluenceClient,
    space: Option<&str>,
    space_id: Option<String>,
) -> anyhow::Result<String> {
    resolve_space_id(client, space, space_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("provide --space or --space-id"))
}

fn read_body(body: Option<String>, body_file: Option<&Path>) -> anyhow::Result<Option<String>> {
    match (body, body_file) {
        (Some(body), None) => Ok(Some(body)),
        (None, Some(path)) => fs::read_to_string(path)
            .with_context(|| format!("failed to read body file `{}`", path.display()))
            .map(Some),
        (None, None) => Ok(None),
        (Some(_), Some(_)) => unreachable!("clap prevents --body and --body-file together"),
    }
}

fn status_from_draft(draft: bool) -> ConfluenceContentStatus {
    if draft {
        ConfluenceContentStatus::Draft
    } else {
        ConfluenceContentStatus::Current
    }
}

impl From<BodyRepresentation> for ConfluenceBodyRepresentation {
    fn from(value: BodyRepresentation) -> Self {
        match value {
            BodyRepresentation::Storage => Self::Storage,
            BodyRepresentation::Wiki => Self::Wiki,
            BodyRepresentation::AtlasDocFormat => Self::AtlasDocFormat,
        }
    }
}

fn print_search_results(
    results: &[ConfluenceSearchResult],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(results),
        OutputFormat::Keys => {
            for result in results {
                if let Some(id) = result
                    .content
                    .as_ref()
                    .and_then(|content| content.id.as_ref())
                {
                    println!("{id}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,type,title,status,url");
            for result in results {
                let content = result.content.as_ref();
                println!(
                    "{},{},{},{},{}",
                    csv_cell(
                        content
                            .and_then(|content| content.id.as_deref())
                            .unwrap_or_default()
                    ),
                    csv_cell(
                        content
                            .and_then(|content| content.content_type.as_deref())
                            .unwrap_or_default()
                    ),
                    csv_cell(search_title(result)),
                    csv_cell(
                        content
                            .and_then(|content| content.status.as_deref())
                            .unwrap_or_default()
                    ),
                    csv_cell(result.url.as_deref().unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<14} {:<14} {:<12} TITLE", "ID", "TYPE", "STATUS");
            for result in results {
                let content = result.content.as_ref();
                println!(
                    "{:<14} {:<14} {:<12} {}",
                    content
                        .and_then(|content| content.id.as_deref())
                        .unwrap_or("-"),
                    content
                        .and_then(|content| content.content_type.as_deref())
                        .unwrap_or("-"),
                    content
                        .and_then(|content| content.status.as_deref())
                        .unwrap_or("-"),
                    search_title(result)
                );
            }
            Ok(())
        }
    }
}

fn search_title(result: &ConfluenceSearchResult) -> &str {
    result
        .title
        .as_deref()
        .or_else(|| {
            result
                .content
                .as_ref()
                .and_then(|content| content.title.as_deref())
        })
        .unwrap_or("-")
}

fn print_attachments(
    attachments: &[ConfluenceAttachment],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(attachments),
        OutputFormat::Keys => {
            for attachment in attachments {
                if let Some(id) = &attachment.id {
                    println!("{id}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,title,status,media_type,file_size,download_link");
            for attachment in attachments {
                println!(
                    "{},{},{},{},{},{}",
                    csv_cell(attachment.id.as_deref().unwrap_or_default()),
                    csv_cell(attachment.title.as_deref().unwrap_or_default()),
                    csv_cell(attachment.status.as_deref().unwrap_or_default()),
                    csv_cell(attachment.media_type.as_deref().unwrap_or_default()),
                    csv_cell(&attachment.file_size.unwrap_or_default().to_string()),
                    csv_cell(attachment.download_link.as_deref().unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<14} {:<18} {:<10} TITLE", "ID", "MEDIA TYPE", "SIZE");
            for attachment in attachments {
                println!(
                    "{:<14} {:<18} {:<10} {}",
                    attachment.id.as_deref().unwrap_or("-"),
                    attachment.media_type.as_deref().unwrap_or("-"),
                    attachment
                        .file_size
                        .map(|size| size.to_string())
                        .as_deref()
                        .unwrap_or("-"),
                    attachment.title.as_deref().unwrap_or("-")
                );
            }
            Ok(())
        }
    }
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

fn print_blog_posts(posts: &[ConfluenceBlogPost], global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(posts),
        OutputFormat::Keys => {
            for post in posts {
                if let Some(id) = &post.id {
                    println!("{id}");
                }
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,title,status,space_id,version");
            for post in posts {
                println!(
                    "{},{},{},{},{}",
                    csv_cell(post.id.as_deref().unwrap_or_default()),
                    csv_cell(post.title.as_deref().unwrap_or_default()),
                    csv_cell(post.status.as_deref().unwrap_or_default()),
                    csv_cell(post.space_id.as_deref().unwrap_or_default()),
                    csv_cell(&blog_post_version(post).unwrap_or_default())
                );
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{:<14} {:<12} {:<10} TITLE", "ID", "STATUS", "VERSION");
            for post in posts {
                println!(
                    "{:<14} {:<12} {:<10} {}",
                    post.id.as_deref().unwrap_or("-"),
                    post.status.as_deref().unwrap_or("-"),
                    blog_post_version(post).as_deref().unwrap_or("-"),
                    post.title.as_deref().unwrap_or("-")
                );
            }
            Ok(())
        }
    }
}

fn print_blog_post(post: &ConfluenceBlogPost, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(post),
        OutputFormat::Keys => {
            if let Some(id) = &post.id {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!("id,title,status,space_id,version");
            println!(
                "{},{},{},{},{}",
                csv_cell(post.id.as_deref().unwrap_or_default()),
                csv_cell(post.title.as_deref().unwrap_or_default()),
                csv_cell(post.status.as_deref().unwrap_or_default()),
                csv_cell(post.space_id.as_deref().unwrap_or_default()),
                csv_cell(&blog_post_version(post).unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("ID: {}", post.id.as_deref().unwrap_or("-"));
            println!("Title: {}", post.title.as_deref().unwrap_or("-"));
            println!("Status: {}", post.status.as_deref().unwrap_or("-"));
            println!("Space ID: {}", post.space_id.as_deref().unwrap_or("-"));
            println!(
                "Version: {}",
                blog_post_version(post).as_deref().unwrap_or("-")
            );
            Ok(())
        }
    }
}

fn blog_post_version(post: &ConfluenceBlogPost) -> Option<String> {
    post.version
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
