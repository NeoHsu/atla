use anyhow::Context;
use atla_core::markdown;
use atla_core::{
    ConfluenceAttachment, ConfluenceAttachmentSearch, ConfluenceAttachmentUpload,
    ConfluenceBlogPost, ConfluenceBlogPostCreate, ConfluenceBlogPostSearch,
    ConfluenceBlogPostUpdate, ConfluenceBodyRepresentation, ConfluenceClient, ConfluenceComment,
    ConfluenceCommentCreate, ConfluenceCommentPage, ConfluenceCommentSearch, ConfluenceContentNode,
    ConfluenceContentStatus, ConfluenceContentTreeSearch, ConfluenceLabelPage,
    ConfluenceLabelSearch, ConfluencePage, ConfluencePageCopy, ConfluencePageCreate,
    ConfluencePageSearch, ConfluencePageUpdate, ConfluenceSearch, ConfluenceSearchResult,
    ConfluenceSpace, ConfluenceSpaceCreate, ConfluenceSpaceSearch, ConfluenceSpaceUpdate,
};
use std::fs;
use std::path::Path;

use crate::cli::{
    AttachmentAction, AttachmentCommand, BlogAction, BlogCommand, BlogCommentAction,
    BlogLabelAction, BodyRepresentation, ConfluenceCommand, ConfluenceResource, ContentViewFormat,
    GlobalArgs, OutputFormat, PageAction, PageCommand, PageCommentAction, PageLabelAction,
    SpaceAction, SpaceCommand,
};
use crate::context::AppContext;
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
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
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

    let client = ctx.confluence_client()?;
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceAttachmentSearch {
                page_id,
                filename: filename.filter(|s| !s.is_empty()),
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

            let client = ctx.confluence_client()?;
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
        AttachmentAction::View { attachment_id } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/attachments/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    attachment_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let attachment = client
                .get_attachment(&attachment_id)
                .await
                .with_context(|| {
                    format!(
                        "failed to load Confluence attachment `{attachment_id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_attachment(&attachment, global)?;
        }
        AttachmentAction::Upload {
            page_id,
            file,
            comment,
            minor_edit,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
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

            let client = ctx.confluence_client()?;
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
        AttachmentAction::Download {
            attachment_id,
            output,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/attachments/{} then download its file using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    attachment_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let download = client
                .download_attachment(&attachment_id, output.as_deref())
                .await
                .with_context(|| {
                    format!(
                        "failed to download Confluence attachment `{attachment_id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_attachment_download(
                &download.path.display().to_string(),
                download.bytes,
                global,
            )?;
        }
        AttachmentAction::Delete {
            attachment_id,
            purge,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/api/v2/attachments/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    attachment_id
                );
                return Ok(());
            }
            if !yes {
                anyhow::bail!("refusing to delete attachment `{attachment_id}` without --yes");
            }

            let client = ctx.confluence_client()?;
            client
                .delete_attachment(&attachment_id, purge)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete Confluence attachment `{attachment_id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("attachment", &attachment_id, global)?;
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let representation = confluence_body_representation(representation)?;

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
            let client = ctx.confluence_client()?;
            let space_id = resolve_required_space_id(&client, space.as_deref(), space_id).await?;
            let post = client
                .create_blog_post(&ConfluenceBlogPostCreate {
                    space_id,
                    title,
                    body,
                    representation,
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
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

            let client = ctx.confluence_client()?;
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/blogposts/{}",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let post = client.get_blog_post(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence blog post `{id}` from {}",
                    client.instance_url()
                )
            })?;

            print_blog_post(&post, global)?;
        }
        BlogAction::Update {
            id,
            title,
            body,
            body_file,
            representation,
            version,
            message,
            draft,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let representation = confluence_body_representation(representation)?;

            if global.dry_run {
                println!(
                    "Would PUT {}/wiki/api/v2/blogposts/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let existing = client.get_blog_post(&id).await.with_context(|| {
                format!(
                    "failed to load Confluence blog post `{id}` from {}",
                    client.instance_url()
                )
            })?;
            let body = read_body(body, body_file.as_deref())?
                .or_else(|| existing.body.clone())
                .ok_or_else(|| anyhow::anyhow!("provide --body or --body-file"))?;
            let title = title
                .or(existing.title)
                .ok_or_else(|| anyhow::anyhow!("blog post `{id}` did not include a title"))?;
            let next_version = version
                .or_else(|| {
                    existing
                        .version
                        .as_ref()
                        .and_then(|version| version.number)
                        .map(|number| number + 1)
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("blog post `{id}` did not include a version; pass --version")
                })?;
            let post = client
                .update_blog_post(&ConfluenceBlogPostUpdate {
                    id: id.clone(),
                    status: status_from_draft(draft),
                    title,
                    space_id: existing.space_id,
                    body,
                    representation,
                    version: next_version,
                    message,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to update Confluence blog post `{id}` from {}",
                        client.instance_url()
                    )
                })?;

            print_blog_post(&post, global)?;
        }
        BlogAction::Delete {
            id,
            purge,
            draft,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/api/v2/blogposts/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                return Ok(());
            }
            if !yes {
                anyhow::bail!("refusing to delete blog post `{id}` without --yes");
            }

            let client = ctx.confluence_client()?;
            client
                .delete_blog_post(&id, purge, draft)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete Confluence blog post `{id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("blogPost", &id, global)?;
        }
        BlogAction::Label { action } => {
            run_blog_label(action, global).await?;
        }
        BlogAction::Comment { action } => {
            run_blog_comment(action, global).await?;
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

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

            let (body, representation) =
                prepare_optional_body(read_body(body, body_file.as_deref())?, representation)?;
            let client = ctx.confluence_client()?;
            let space_id = resolve_required_space_id(&client, space.as_deref(), space_id).await?;
            let page = client
                .create_page(&ConfluencePageCreate {
                    space_id,
                    title,
                    parent_id: parent,
                    body,
                    representation,
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
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

            let client = ctx.confluence_client()?;
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
        PageAction::View { id, web, format } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/pages/{}",
                    profile.instance.trim_end_matches('/'),
                    id
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            if web {
                open_web_url(&format!(
                    "{}/wiki/pages/viewpage.action?pageId={}",
                    profile.instance.trim_end_matches('/'),
                    id
                ))?;
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let page = client
                .get_page_with_body_format(&id, format.and_then(view_format_body_representation))
                .await
                .with_context(|| {
                    format!(
                        "failed to load Confluence page `{id}` from {}",
                        client.instance_url()
                    )
                })?;

            if matches!(format, Some(ContentViewFormat::Markdown)) {
                print_page_body_markdown(&page)?;
            } else if format.is_some() {
                print_page_body(&page)?;
            } else {
                print_page(&page, global)?;
            }
        }
        PageAction::Children { id, depth, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceContentTreeSearch {
                page_id: id,
                limit: limit.clamp(1, 250),
                depth: depth.map(|depth| depth.clamp(1, 100)),
            };

            if global.dry_run {
                let endpoint = if let Some(depth) = search.depth {
                    format!(
                        "{}/wiki/api/v2/pages/{}/descendants?limit={}&depth={depth}",
                        profile.instance.trim_end_matches('/'),
                        search.page_id,
                        search.limit
                    )
                } else {
                    format!(
                        "{}/wiki/api/v2/pages/{}/direct-children?limit={}",
                        profile.instance.trim_end_matches('/'),
                        search.page_id,
                        search.limit
                    )
                };
                println!("Would GET {endpoint} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let children = client.list_page_children(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence page children for `{}` from {}",
                    search.page_id,
                    client.instance_url()
                )
            })?;

            print_content_nodes(&children.results, global)?;
        }
        PageAction::Copy {
            source_id,
            title,
            space,
            space_id,
            parent,
            root_level,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                if let Some(space) = &space {
                    println!(
                        "Would GET {}/wiki/api/v2/spaces?keys={space}&limit=1 using profile `{profile_name}`",
                        profile.instance.trim_end_matches('/')
                    );
                }
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}?body-format=storage using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    source_id
                );
                println!(
                    "Would POST {}/wiki/api/v2/pages using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let resolved_space_id = resolve_space_id(&client, space.as_deref(), space_id).await?;
            let page = client
                .copy_page(&ConfluencePageCopy {
                    source_id: source_id.clone(),
                    title,
                    space_id: resolved_space_id,
                    parent_id: parent,
                    root_level,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to copy Confluence page `{source_id}` from {}",
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

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

            let client = ctx.confluence_client()?;
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

            let (body, representation) = prepare_required_body(
                read_body(body, body_file.as_deref())?,
                representation,
                "page body update and move require --body or --body-file",
            )?;
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
                    representation,
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

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

            let client = ctx.confluence_client()?;
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
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

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

            let client = ctx.confluence_client()?;
            let page = client.move_page(&id, &parent).await.with_context(|| {
                format!(
                    "failed to move Confluence page `{id}` under `{parent}` from {}",
                    client.instance_url()
                )
            })?;
            print_page(&page, global)?;
        }
        PageAction::Label { action } => {
            run_page_label(action, global).await?;
        }
        PageAction::Comment { action } => {
            run_page_comment(action, global).await?;
        }
    }

    Ok(())
}

async fn run_space(command: SpaceCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    match command.action {
        SpaceAction::List { key, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
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

            let client = ctx.confluence_client()?;
            let page = client.list_spaces(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence spaces from {}",
                    client.instance_url()
                )
            })?;

            print_spaces(&page.results, global)?;
        }
        SpaceAction::View { key } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                let url = format!(
                    "{}/wiki/api/v2/spaces?keys={}&limit=1",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                println!("Would GET {url} using profile `{profile_name}`");
                return Ok(());
            }

            let client = ctx.confluence_client()?;
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
        SpaceAction::Create {
            name,
            key,
            alias,
            description,
            description_file,
            private,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            if key.is_none() && alias.is_none() {
                anyhow::bail!("provide --key or --alias");
            }
            let description = read_body(description, description_file.as_deref())?;
            let create = ConfluenceSpaceCreate {
                key,
                alias,
                name,
                description,
                private,
            };

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/api/v2/spaces using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let space = client.create_space(&create).await.with_context(|| {
                format!(
                    "failed to create Confluence space in {}",
                    client.instance_url()
                )
            })?;

            print_space(&space, global)?;
        }
        SpaceAction::Update {
            key,
            name,
            description,
            description_file,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let description = read_body(description, description_file.as_deref())?;
            if name.is_none() && description.is_none() {
                anyhow::bail!("provide --name, --description, or --description-file");
            }
            let update = ConfluenceSpaceUpdate {
                key,
                name,
                description,
            };

            if global.dry_run {
                println!(
                    "Would PUT {}/wiki/rest/api/space/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    update.key
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let space = client.update_space(&update).await.with_context(|| {
                format!(
                    "failed to update Confluence space `{}` from {}",
                    update.key,
                    client.instance_url()
                )
            })?;

            print_space(&space, global)?;
        }
        SpaceAction::Delete { key, yes } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/rest/api/space/{} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    key
                );
                return Ok(());
            }
            if !yes {
                anyhow::bail!("refusing to delete space `{key}` without --yes");
            }

            let client = ctx.confluence_client()?;
            client.delete_space(&key).await.with_context(|| {
                format!(
                    "failed to delete Confluence space `{key}` from {}",
                    client.instance_url()
                )
            })?;
            print_deleted("space", &key, global)?;
        }
    }

    Ok(())
}

async fn run_page_label(action: PageLabelAction, global: &GlobalArgs) -> anyhow::Result<()> {
    match action {
        PageLabelAction::List {
            page_id,
            prefix,
            limit,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceLabelSearch {
                content_id: page_id,
                prefix,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}/labels?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.content_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let labels = client.list_page_labels(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence page labels from {}",
                    client.instance_url()
                )
            })?;
            print_labels(&labels, global)?;
        }
        PageLabelAction::Add { page_id, labels } => {
            if labels.is_empty() {
                anyhow::bail!("provide at least one label");
            }
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/rest/api/content/{}/label using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    page_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let labels = client
                .add_page_labels(&page_id, &labels)
                .await
                .with_context(|| {
                    format!(
                        "failed to add Confluence page labels from {}",
                        client.instance_url()
                    )
                })?;
            print_labels(&labels, global)?;
        }
        PageLabelAction::Remove { page_id, label } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/rest/api/content/{}/label?name={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    page_id,
                    label
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            client
                .remove_page_label(&page_id, &label)
                .await
                .with_context(|| {
                    format!(
                        "failed to remove Confluence page label `{label}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("label", &label, global)?;
        }
    }

    Ok(())
}

async fn run_page_comment(action: PageCommentAction, global: &GlobalArgs) -> anyhow::Result<()> {
    match action {
        PageCommentAction::List { page_id, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceCommentSearch {
                content_id: page_id,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/pages/{}/footer-comments?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.content_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let comments = client.list_page_comments(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence page comments from {}",
                    client.instance_url()
                )
            })?;
            print_comments(&comments, global)?;
        }
        PageCommentAction::Add {
            page_id,
            body,
            body_file,
            parent,
            representation,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let (body, representation) = prepare_required_body(
                read_body(body, body_file.as_deref())?,
                representation,
                "missing comment body",
            )?;

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/api/v2/footer-comments using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let comment = client
                .add_page_comment(&ConfluenceCommentCreate {
                    content_id: page_id,
                    parent_comment_id: parent,
                    body,
                    representation,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to add Confluence page comment from {}",
                        client.instance_url()
                    )
                })?;
            print_comment(&comment, global)?;
        }
        PageCommentAction::Delete {
            page_id,
            comment_id,
            yes,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if !yes && !global.dry_run {
                anyhow::bail!("refusing to delete Confluence comment `{comment_id}` without --yes");
            }

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/api/v2/footer-comments/{} for page `{page_id}` using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    comment_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            client
                .delete_page_comment(&comment_id)
                .await
                .with_context(|| {
                    format!(
                        "failed to delete Confluence comment `{comment_id}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("comment", &comment_id, global)?;
        }
    }

    Ok(())
}

async fn run_blog_label(action: BlogLabelAction, global: &GlobalArgs) -> anyhow::Result<()> {
    match action {
        BlogLabelAction::List {
            blog_id,
            prefix,
            limit,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceLabelSearch {
                content_id: blog_id,
                prefix,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/blogposts/{}/labels?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.content_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let labels = client.list_blog_labels(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence blog labels from {}",
                    client.instance_url()
                )
            })?;
            print_labels(&labels, global)?;
        }
        BlogLabelAction::Add { blog_id, labels } => {
            if labels.is_empty() {
                anyhow::bail!("provide at least one label");
            }
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/rest/api/content/{}/label using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    blog_id
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let labels = client
                .add_blog_labels(&blog_id, &labels)
                .await
                .with_context(|| {
                    format!(
                        "failed to add Confluence blog labels from {}",
                        client.instance_url()
                    )
                })?;
            print_labels(&labels, global)?;
        }
        BlogLabelAction::Remove { blog_id, label } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();

            if global.dry_run {
                println!(
                    "Would DELETE {}/wiki/rest/api/content/{}/label?name={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    blog_id,
                    label
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            client
                .remove_blog_label(&blog_id, &label)
                .await
                .with_context(|| {
                    format!(
                        "failed to remove Confluence blog label `{label}` from {}",
                        client.instance_url()
                    )
                })?;
            print_deleted("label", &label, global)?;
        }
    }

    Ok(())
}

async fn run_blog_comment(action: BlogCommentAction, global: &GlobalArgs) -> anyhow::Result<()> {
    match action {
        BlogCommentAction::List { blog_id, limit } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let search = ConfluenceCommentSearch {
                content_id: blog_id,
                limit: limit.clamp(1, 250),
            };

            if global.dry_run {
                println!(
                    "Would GET {}/wiki/api/v2/blogposts/{}/footer-comments?limit={} using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/'),
                    search.content_id,
                    search.limit
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let comments = client.list_blog_comments(&search).await.with_context(|| {
                format!(
                    "failed to list Confluence blog comments from {}",
                    client.instance_url()
                )
            })?;
            print_comments(&comments, global)?;
        }
        BlogCommentAction::Add {
            blog_id,
            body,
            body_file,
            parent,
            representation,
        } => {
            let ctx = AppContext::load(global)?;
            let profile_name = ctx.profile_name();
            let profile = ctx.profile();
            let body = read_body(body, body_file.as_deref())?
                .ok_or_else(|| anyhow::anyhow!("missing comment body"))?;
            let representation = confluence_body_representation(representation)?;

            if global.dry_run {
                println!(
                    "Would POST {}/wiki/api/v2/footer-comments using profile `{profile_name}`",
                    profile.instance.trim_end_matches('/')
                );
                return Ok(());
            }

            let client = ctx.confluence_client()?;
            let comment = client
                .add_blog_comment(&ConfluenceCommentCreate {
                    content_id: blog_id,
                    parent_comment_id: parent,
                    body,
                    representation,
                })
                .await
                .with_context(|| {
                    format!(
                        "failed to add Confluence blog comment from {}",
                        client.instance_url()
                    )
                })?;
            print_comment(&comment, global)?;
        }
    }

    Ok(())
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

fn prepare_optional_body(
    body: Option<String>,
    representation: BodyRepresentation,
) -> anyhow::Result<(Option<String>, ConfluenceBodyRepresentation)> {
    match representation {
        BodyRepresentation::Markdown => body
            .map(markdown_body_to_adf)
            .transpose()
            .map(|body| (body, ConfluenceBodyRepresentation::AtlasDocFormat)),
        _ => Ok((body, confluence_body_representation(representation)?)),
    }
}

fn prepare_required_body(
    body: Option<String>,
    representation: BodyRepresentation,
    missing_message: &str,
) -> anyhow::Result<(String, ConfluenceBodyRepresentation)> {
    let (body, representation) = prepare_optional_body(body, representation)?;
    Ok((
        body.ok_or_else(|| anyhow::anyhow!(missing_message.to_owned()))?,
        representation,
    ))
}

fn markdown_body_to_adf(body: String) -> anyhow::Result<String> {
    serde_json::to_string(&markdown::markdown_to_adf(&body))
        .context("failed to encode Markdown body as Atlas Doc Format")
}

fn confluence_body_representation(
    representation: BodyRepresentation,
) -> anyhow::Result<ConfluenceBodyRepresentation> {
    match representation {
        BodyRepresentation::Storage => Ok(ConfluenceBodyRepresentation::Storage),
        BodyRepresentation::Wiki => Ok(ConfluenceBodyRepresentation::Wiki),
        BodyRepresentation::AtlasDocFormat => Ok(ConfluenceBodyRepresentation::AtlasDocFormat),
        BodyRepresentation::Markdown => {
            anyhow::bail!("--representation markdown is supported for pages and page comments only")
        }
    }
}

fn status_from_draft(draft: bool) -> ConfluenceContentStatus {
    if draft {
        ConfluenceContentStatus::Draft
    } else {
        ConfluenceContentStatus::Current
    }
}

fn view_format_body_representation(
    format: ContentViewFormat,
) -> Option<ConfluenceBodyRepresentation> {
    match format {
        ContentViewFormat::Markdown | ContentViewFormat::AtlasDocFormat => {
            Some(ConfluenceBodyRepresentation::AtlasDocFormat)
        }
        ContentViewFormat::Storage => Some(ConfluenceBodyRepresentation::Storage),
    }
}

fn print_page_body(page: &ConfluencePage) -> anyhow::Result<()> {
    let body = page
        .body
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("page did not include a body"))?;
    println!("{body}");
    Ok(())
}

fn print_page_body_markdown(page: &ConfluencePage) -> anyhow::Result<()> {
    let body = page
        .body
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("page did not include a body"))?;
    let markdown = serde_json::from_str::<serde_json::Value>(body)
        .map(|adf| markdown::adf_to_markdown(&adf))
        .unwrap_or_else(|_| body.to_owned());
    println!("{markdown}");
    Ok(())
}

fn open_web_url(url: &str) -> anyhow::Result<()> {
    let command = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "xdg-open"
    };
    let status = if cfg!(target_os = "windows") {
        std::process::Command::new(command)
            .args(["/C", "start", "", url])
            .status()
    } else {
        std::process::Command::new(command).arg(url).status()
    };

    match status {
        Ok(status) if status.success() => Ok(()),
        _ => {
            println!("{url}");
            Ok(())
        }
    }
}

fn print_search_results(
    results: &[ConfluenceSearchResult],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        results,
        results
            .iter()
            .filter_map(|result| {
                result
                    .content
                    .as_ref()
                    .and_then(|content| content.id.clone())
            })
            .collect(),
        &["id", "type", "title", "status", "url"],
        results
            .iter()
            .map(|result| {
                let content = result.content.as_ref();
                vec![
                    content
                        .and_then(|content| content.id.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                    content
                        .and_then(|content| content.content_type.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                    search_title(result).to_owned(),
                    content
                        .and_then(|content| content.status.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                    result.url.as_deref().unwrap_or("-").to_owned(),
                ]
            })
            .collect(),
        None,
    )
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

fn print_attachment(attachment: &ConfluenceAttachment, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(attachment),
        OutputFormat::Keys => {
            if let Some(id) = &attachment.id {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Csv => {
            println!(
                "id,title,status,page_id,blog_post_id,media_type,media_type_description,file_id,file_size,version,webui_link,download_link"
            );
            println!(
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                output::csv_cell(attachment.id.as_deref().unwrap_or_default()),
                output::csv_cell(attachment.title.as_deref().unwrap_or_default()),
                output::csv_cell(attachment.status.as_deref().unwrap_or_default()),
                output::csv_cell(attachment.page_id.as_deref().unwrap_or_default()),
                output::csv_cell(attachment.blog_post_id.as_deref().unwrap_or_default()),
                output::csv_cell(attachment.media_type.as_deref().unwrap_or_default()),
                output::csv_cell(
                    attachment
                        .media_type_description
                        .as_deref()
                        .unwrap_or_default()
                ),
                output::csv_cell(attachment.file_id.as_deref().unwrap_or_default()),
                output::csv_cell(&attachment.file_size.unwrap_or_default().to_string()),
                output::csv_cell(&attachment_version(attachment).unwrap_or_default()),
                output::csv_cell(attachment.webui_link.as_deref().unwrap_or_default()),
                output::csv_cell(attachment.download_link.as_deref().unwrap_or_default())
            );
            Ok(())
        }
        OutputFormat::Table => {
            println!("ID: {}", attachment.id.as_deref().unwrap_or("-"));
            println!("Title: {}", attachment.title.as_deref().unwrap_or("-"));
            println!("Status: {}", attachment.status.as_deref().unwrap_or("-"));
            println!("Page ID: {}", attachment.page_id.as_deref().unwrap_or("-"));
            println!(
                "Blog Post ID: {}",
                attachment.blog_post_id.as_deref().unwrap_or("-")
            );
            println!(
                "Media Type: {}",
                attachment.media_type.as_deref().unwrap_or("-")
            );
            println!(
                "Media Type Description: {}",
                attachment.media_type_description.as_deref().unwrap_or("-")
            );
            println!("File ID: {}", attachment.file_id.as_deref().unwrap_or("-"));
            println!(
                "File Size: {}",
                attachment
                    .file_size
                    .map(|size| size.to_string())
                    .as_deref()
                    .unwrap_or("-")
            );
            println!(
                "Version: {}",
                attachment_version(attachment).as_deref().unwrap_or("-")
            );
            if let Some(link) = &attachment.webui_link {
                println!("Web UI: {link}");
            }
            if let Some(link) = &attachment.download_link {
                println!("Download: {link}");
            }
            Ok(())
        }
    }
}

fn attachment_version(attachment: &ConfluenceAttachment) -> Option<String> {
    attachment
        .version
        .as_ref()
        .and_then(|version| version.number)
        .map(|number| number.to_string())
}

fn print_attachments(
    attachments: &[ConfluenceAttachment],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        attachments,
        attachments
            .iter()
            .filter_map(|attachment| attachment.id.clone())
            .collect(),
        &[
            "id",
            "title",
            "status",
            "media_type",
            "file_size",
            "download_link",
        ],
        attachments
            .iter()
            .map(|attachment| {
                vec![
                    attachment.id.as_deref().unwrap_or("-").to_owned(),
                    attachment.title.as_deref().unwrap_or("-").to_owned(),
                    attachment.status.as_deref().unwrap_or("-").to_owned(),
                    attachment.media_type.as_deref().unwrap_or("-").to_owned(),
                    attachment
                        .file_size
                        .map(|size| size.to_string())
                        .unwrap_or("-".to_owned()),
                    attachment
                        .download_link
                        .as_deref()
                        .unwrap_or("-")
                        .to_owned(),
                ]
            })
            .collect(),
        None,
    )
}

fn print_attachment_download(path: &str, bytes: u64, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(&serde_json::json!({
            "path": path,
            "bytes": bytes
        })),
        OutputFormat::Keys => {
            println!("{path}");
            Ok(())
        }
        OutputFormat::Csv => {
            println!("path,bytes");
            println!("{},{}", output::csv_cell(path), bytes);
            Ok(())
        }
        OutputFormat::Table => {
            println!("Downloaded: {path}");
            println!("Bytes: {bytes}");
            Ok(())
        }
    }
}

fn print_deleted(kind: &str, id: &str, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => output::print_json(&serde_json::json!({
            "type": kind,
            "id": id,
            "deleted": true
        })),
        OutputFormat::Keys => {
            println!("{id}");
            Ok(())
        }
        OutputFormat::Csv => {
            println!("type,id,deleted");
            println!("{},{},true", output::csv_cell(kind), output::csv_cell(id));
            Ok(())
        }
        OutputFormat::Table => {
            println!("Deleted {kind}: {id}");
            Ok(())
        }
    }
}

fn print_labels(page: &ConfluenceLabelPage, global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        page,
        page.results
            .iter()
            .filter_map(|label| label.name.clone())
            .collect(),
        &["id", "name", "prefix"],
        page.results
            .iter()
            .map(|label| {
                vec![
                    label.id.as_deref().unwrap_or("-").to_owned(),
                    label.name.as_deref().unwrap_or("-").to_owned(),
                    label.prefix.as_deref().unwrap_or("-").to_owned(),
                ]
            })
            .collect(),
        None,
    )
}

fn print_comments(page: &ConfluenceCommentPage, global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        page,
        page.results
            .iter()
            .filter_map(|comment| comment.id.clone())
            .collect(),
        &["id", "page_id", "status", "version", "body"],
        page.results
            .iter()
            .map(|comment| {
                vec![
                    comment.id.as_deref().unwrap_or("-").to_owned(),
                    comment.page_id.as_deref().unwrap_or("-").to_owned(),
                    comment.status.as_deref().unwrap_or("-").to_owned(),
                    comment_version(comment).unwrap_or("-".to_owned()),
                    comment.body.as_deref().unwrap_or("-").replace('\n', " "),
                ]
            })
            .collect(),
        None,
    )
}

fn print_comment(comment: &ConfluenceComment, global: &GlobalArgs) -> anyhow::Result<()> {
    print_comments(
        &ConfluenceCommentPage {
            results: vec![comment.clone()],
        },
        global,
    )
}

fn comment_version(comment: &ConfluenceComment) -> Option<String> {
    comment
        .version
        .as_ref()
        .and_then(|version| version.number)
        .map(|number| number.to_string())
}

fn print_content_nodes(nodes: &[ConfluenceContentNode], global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        nodes,
        nodes.iter().filter_map(|node| node.id.clone()).collect(),
        &[
            "id",
            "type",
            "title",
            "status",
            "space_id",
            "parent_id",
            "depth",
            "child_position",
        ],
        nodes
            .iter()
            .map(|node| {
                vec![
                    node.id.as_deref().unwrap_or("-").to_owned(),
                    node.content_type.as_deref().unwrap_or("-").to_owned(),
                    node.title.as_deref().unwrap_or("-").to_owned(),
                    node.status.as_deref().unwrap_or("-").to_owned(),
                    node.space_id.as_deref().unwrap_or("-").to_owned(),
                    node.parent_id.as_deref().unwrap_or("-").to_owned(),
                    node.depth
                        .map(|depth| depth.to_string())
                        .unwrap_or_else(|| "-".to_owned()),
                    node.child_position
                        .map(|position| position.to_string())
                        .unwrap_or_else(|| "-".to_owned()),
                ]
            })
            .collect(),
        None,
    )
}

fn print_pages(pages: &[ConfluencePage], global: &GlobalArgs) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        pages,
        pages.iter().filter_map(|page| page.id.clone()).collect(),
        &["id", "title", "status", "space_id", "parent_id", "version"],
        pages
            .iter()
            .map(|page| {
                vec![
                    page.id.as_deref().unwrap_or("-").to_owned(),
                    page.title.as_deref().unwrap_or("-").to_owned(),
                    page.status.as_deref().unwrap_or("-").to_owned(),
                    page.space_id.as_deref().unwrap_or("-").to_owned(),
                    page.parent_id.as_deref().unwrap_or("-").to_owned(),
                    page_version(page).unwrap_or("-".to_owned()),
                ]
            })
            .collect(),
        None,
    )
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
                output::csv_cell(page.id.as_deref().unwrap_or_default()),
                output::csv_cell(page.title.as_deref().unwrap_or_default()),
                output::csv_cell(page.status.as_deref().unwrap_or_default()),
                output::csv_cell(page.space_id.as_deref().unwrap_or_default()),
                output::csv_cell(page.parent_id.as_deref().unwrap_or_default()),
                output::csv_cell(&page_version(page).unwrap_or_default())
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
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        posts,
        posts.iter().filter_map(|post| post.id.clone()).collect(),
        &["id", "title", "status", "space_id", "version"],
        posts
            .iter()
            .map(|post| {
                vec![
                    post.id.as_deref().unwrap_or("-").to_owned(),
                    post.title.as_deref().unwrap_or("-").to_owned(),
                    post.status.as_deref().unwrap_or("-").to_owned(),
                    post.space_id.as_deref().unwrap_or("-").to_owned(),
                    blog_post_version(post).unwrap_or("-".to_owned()),
                ]
            })
            .collect(),
        None,
    )
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
                output::csv_cell(post.id.as_deref().unwrap_or_default()),
                output::csv_cell(post.title.as_deref().unwrap_or_default()),
                output::csv_cell(post.status.as_deref().unwrap_or_default()),
                output::csv_cell(post.space_id.as_deref().unwrap_or_default()),
                output::csv_cell(&blog_post_version(post).unwrap_or_default())
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
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        spaces,
        spaces
            .iter()
            .filter_map(|space| space.key.clone())
            .collect(),
        &["key", "name", "type", "status", "id", "homepage_id"],
        spaces
            .iter()
            .map(|space| {
                vec![
                    space.key.as_deref().unwrap_or("-").to_owned(),
                    space.name.as_deref().unwrap_or("-").to_owned(),
                    space.space_type.as_deref().unwrap_or("-").to_owned(),
                    space.status.as_deref().unwrap_or("-").to_owned(),
                    space.id.as_deref().unwrap_or("-").to_owned(),
                    space.homepage_id.as_deref().unwrap_or("-").to_owned(),
                ]
            })
            .collect(),
        None,
    )
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
                output::csv_cell(space.key.as_deref().unwrap_or_default()),
                output::csv_cell(space.name.as_deref().unwrap_or_default()),
                output::csv_cell(space.space_type.as_deref().unwrap_or_default()),
                output::csv_cell(space.status.as_deref().unwrap_or_default()),
                output::csv_cell(space.id.as_deref().unwrap_or_default()),
                output::csv_cell(space.homepage_id.as_deref().unwrap_or_default())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_markdown_body_to_adf_write_body() {
        let (body, representation) = prepare_optional_body(
            Some("# Title\n\n**Important**".to_owned()),
            BodyRepresentation::Markdown,
        )
        .expect("convert markdown");

        assert_eq!(representation, ConfluenceBodyRepresentation::AtlasDocFormat);
        let body = body.expect("converted body");
        let adf: serde_json::Value = serde_json::from_str(&body).expect("adf json");
        assert_eq!(adf["type"], "doc");
        assert_eq!(adf["content"][0]["type"], "heading");
    }

    #[test]
    fn rejects_markdown_for_non_page_write_paths() {
        let error = confluence_body_representation(BodyRepresentation::Markdown)
            .expect_err("markdown should require page-specific conversion");

        assert!(
            error
                .to_string()
                .contains("supported for pages and page comments only")
        );
    }
}
