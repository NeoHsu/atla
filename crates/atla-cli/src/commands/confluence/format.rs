use anyhow::Context;
use atla_core::markdown;
use atla_core::{
    ConfluenceAttachment, ConfluenceBlogPost, ConfluenceBodyRepresentation, ConfluenceClient,
    ConfluenceComment, ConfluenceCommentPage, ConfluenceContentNode, ConfluenceContentStatus,
    ConfluenceLabelPage, ConfluencePage, ConfluenceSearchResult, ConfluenceSpace,
};
use std::fs;
use std::path::Path;

use crate::cli::{BodyRepresentation, ContentViewFormat, GlobalArgs, OutputFormat};
use crate::output;

pub(super) async fn resolve_space_id(
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

pub(super) async fn resolve_required_space_id(
    client: &ConfluenceClient,
    space: Option<&str>,
    space_id: Option<String>,
) -> anyhow::Result<String> {
    resolve_space_id(client, space, space_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("provide --space or --space-id"))
}

pub(super) fn read_body(
    body: Option<String>,
    body_file: Option<&Path>,
) -> anyhow::Result<Option<String>> {
    match (body, body_file) {
        (Some(body), None) => Ok(Some(body)),
        (None, Some(path)) => fs::read_to_string(path)
            .with_context(|| format!("failed to read body file `{}`", path.display()))
            .map(Some),
        (None, None) => Ok(None),
        (Some(_), Some(_)) => unreachable!("clap prevents --body and --body-file together"),
    }
}

pub(super) fn prepare_optional_body(
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

pub(super) fn prepare_required_body(
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

pub(super) fn markdown_body_to_adf(body: String) -> anyhow::Result<String> {
    serde_json::to_string(&markdown::markdown_to_adf(&body))
        .context("failed to encode Markdown body as Atlas Doc Format")
}

pub(super) fn confluence_body_representation(
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

pub(super) fn status_from_draft(draft: bool) -> ConfluenceContentStatus {
    if draft {
        ConfluenceContentStatus::Draft
    } else {
        ConfluenceContentStatus::Current
    }
}

pub(super) fn view_format_body_representation(
    format: ContentViewFormat,
) -> Option<ConfluenceBodyRepresentation> {
    match format {
        ContentViewFormat::Markdown | ContentViewFormat::AtlasDocFormat => {
            Some(ConfluenceBodyRepresentation::AtlasDocFormat)
        }
        ContentViewFormat::Storage => Some(ConfluenceBodyRepresentation::Storage),
    }
}

pub(super) fn print_page_body(page: &ConfluencePage) -> anyhow::Result<()> {
    let body = page
        .body
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("page did not include a body"))?;
    println!("{body}");
    Ok(())
}

pub(super) fn print_page_body_markdown(page: &ConfluencePage) -> anyhow::Result<()> {
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

pub(super) fn open_web_url(url: &str) -> anyhow::Result<()> {
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

pub(super) fn print_search_results(
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

pub(super) fn search_title(result: &ConfluenceSearchResult) -> &str {
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

pub(super) fn print_attachment(
    attachment: &ConfluenceAttachment,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
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

pub(super) fn attachment_version(attachment: &ConfluenceAttachment) -> Option<String> {
    attachment
        .version
        .as_ref()
        .and_then(|version| version.number)
        .map(|number| number.to_string())
}

pub(super) fn print_attachments(
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

pub(super) fn print_attachment_download(
    path: &str,
    bytes: u64,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
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

pub(super) fn print_deleted(kind: &str, id: &str, global: &GlobalArgs) -> anyhow::Result<()> {
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

pub(super) fn print_labels(page: &ConfluenceLabelPage, global: &GlobalArgs) -> anyhow::Result<()> {
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

pub(super) fn print_comments(
    page: &ConfluenceCommentPage,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
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

pub(super) fn print_comment(
    comment: &ConfluenceComment,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    print_comments(
        &ConfluenceCommentPage {
            results: vec![comment.clone()],
        },
        global,
    )
}

pub(super) fn comment_version(comment: &ConfluenceComment) -> Option<String> {
    comment
        .version
        .as_ref()
        .and_then(|version| version.number)
        .map(|number| number.to_string())
}

pub(super) fn print_content_nodes(
    nodes: &[ConfluenceContentNode],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
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

pub(super) fn print_pages(pages: &[ConfluencePage], global: &GlobalArgs) -> anyhow::Result<()> {
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

pub(super) fn print_page(page: &ConfluencePage, global: &GlobalArgs) -> anyhow::Result<()> {
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

pub(super) fn page_version(page: &ConfluencePage) -> Option<String> {
    page.version
        .as_ref()
        .and_then(|version| version.number)
        .map(|number| number.to_string())
}

pub(super) fn print_blog_posts(
    posts: &[ConfluenceBlogPost],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
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

pub(super) fn print_blog_post(
    post: &ConfluenceBlogPost,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
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

pub(super) fn blog_post_version(post: &ConfluenceBlogPost) -> Option<String> {
    post.version
        .as_ref()
        .and_then(|version| version.number)
        .map(|number| number.to_string())
}

pub(super) fn print_spaces(spaces: &[ConfluenceSpace], global: &GlobalArgs) -> anyhow::Result<()> {
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

pub(super) fn print_space(space: &ConfluenceSpace, global: &GlobalArgs) -> anyhow::Result<()> {
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
