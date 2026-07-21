use anyhow::Context;
use atla_core::markdown;
use atla_core::{
    ConfluenceAttachment, ConfluenceBlogPost, ConfluenceBodyRepresentation, ConfluenceClient,
    ConfluenceComment, ConfluenceCommentPage, ConfluenceContentNode, ConfluenceContentStatus,
    ConfluenceLabelPage, ConfluencePage, ConfluenceSearchResult, ConfluenceSpace, JiraClient,
    JiraUser,
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
        .get_space(space_key)
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

pub(super) async fn markdown_to_adf_options_for_body(
    body: Option<&str>,
    representation: BodyRepresentation,
    numbered_table_rows: bool,
    mention_args: &[String],
    resolve_mentions: bool,
    jira_client: Option<&JiraClient>,
) -> anyhow::Result<markdown::MarkdownToAdfOptions> {
    if representation != BodyRepresentation::Markdown {
        if numbered_table_rows {
            anyhow::bail!("--numbered-table-rows requires --representation markdown");
        }
        if !mention_args.is_empty() {
            anyhow::bail!("--mention requires --representation markdown");
        }
        if resolve_mentions {
            anyhow::bail!("--resolve-mentions requires --representation markdown");
        }
    }

    let mut mentions = mention_args
        .iter()
        .map(|raw| parse_markdown_mention_arg(raw))
        .collect::<anyhow::Result<Vec<_>>>()?;

    if representation == BodyRepresentation::Markdown
        && resolve_mentions
        && let Some(body) = body
    {
        let client = jira_client.ok_or_else(|| {
            anyhow::anyhow!("--resolve-mentions requires an Atlassian API client")
        })?;
        for candidate in markdown::markdown_mention_candidates(body) {
            if mentions
                .iter()
                .any(|mention| mention_matches(mention, &candidate))
            {
                continue;
            }
            let users = client
                .search_users(&candidate)
                .await
                .with_context(|| format!("failed to resolve mention `@{candidate}`"))?;
            match resolve_mention_user(&candidate, users) {
                MentionUserResolution::Resolved(mention) => mentions.push(mention),
                MentionUserResolution::NotFound => {
                    eprintln!(
                        "warning: mention `@{candidate}` was not resolved; leaving it as text"
                    );
                }
                MentionUserResolution::Ambiguous(names) => {
                    let names = names.join(", ");
                    eprintln!(
                        "warning: mention `@{candidate}` matched multiple users ({names}); pass --mention `{candidate}=ACCOUNT_ID` to choose one"
                    );
                }
            }
        }
    }

    Ok(markdown::MarkdownToAdfOptions {
        numbered_table_rows,
        mentions,
    })
}

fn parse_markdown_mention_arg(raw: &str) -> anyhow::Result<markdown::MarkdownMention> {
    let (name, account_id) = raw
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("--mention must use NAME=ACCOUNT_ID"))?;
    let name = name.trim().trim_start_matches('@').trim();
    let account_id = account_id.trim();
    if name.is_empty() || account_id.is_empty() {
        anyhow::bail!("--mention must include both NAME and ACCOUNT_ID");
    }
    Ok(markdown::MarkdownMention {
        text: name.to_owned(),
        account_id: account_id.to_owned(),
    })
}

fn mention_matches(mention: &markdown::MarkdownMention, candidate: &str) -> bool {
    mention.text.trim().eq_ignore_ascii_case(candidate.trim())
}

#[derive(Debug, PartialEq, Eq)]
enum MentionUserResolution {
    Resolved(markdown::MarkdownMention),
    NotFound,
    Ambiguous(Vec<String>),
}

fn resolve_mention_user(query: &str, users: Vec<JiraUser>) -> MentionUserResolution {
    let users = users
        .into_iter()
        .filter(|user| user.active != Some(false))
        .filter(|user| user.account_id.is_some())
        .collect::<Vec<_>>();

    let exact = users
        .iter()
        .filter(|user| {
            user.account_id.as_deref() == Some(query)
                || user
                    .display_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(query))
        })
        .collect::<Vec<_>>();
    if exact.len() == 1 {
        return MentionUserResolution::Resolved(user_to_mention(query, exact[0]));
    }

    match users.as_slice() {
        [user] => MentionUserResolution::Resolved(user_to_mention(query, user)),
        [] => MentionUserResolution::NotFound,
        _ => MentionUserResolution::Ambiguous(
            users
                .iter()
                .map(|user| {
                    let name = user.display_name.as_deref().unwrap_or("unknown");
                    let account_id = user.account_id.as_deref().unwrap_or("unknown-account-id");
                    format!("{name} <{account_id}>")
                })
                .collect(),
        ),
    }
}

fn user_to_mention(query: &str, user: &JiraUser) -> markdown::MarkdownMention {
    markdown::MarkdownMention {
        text: query.trim().trim_start_matches('@').to_owned(),
        account_id: user
            .account_id
            .as_deref()
            .expect("filtered users include account ids")
            .to_owned(),
    }
}

fn looks_like_markdown(body: &str) -> bool {
    let body = body.trim();
    if body.is_empty() || body.starts_with('<') {
        return false;
    }

    body.lines().any(|line| {
        let line = line.trim_start();
        let ordered_marker = line
            .find(|character: char| !character.is_ascii_digit())
            .is_some_and(|index| index > 0 && line[index..].starts_with(". "));
        line.starts_with("# ")
            || line.starts_with("## ")
            || line.starts_with("- ")
            || line.starts_with("* ")
            || line.starts_with("+ ")
            || line.starts_with("> ")
            || line.starts_with("```")
            || line.starts_with("~~~")
            || ordered_marker
    }) || (body.contains("[") && body.contains("]("))
}

pub(super) fn prepare_optional_body_with_options(
    body: Option<String>,
    representation: BodyRepresentation,
    markdown_options: markdown::MarkdownToAdfOptions,
) -> anyhow::Result<(Option<String>, ConfluenceBodyRepresentation)> {
    if matches!(representation, BodyRepresentation::Storage)
        && body.as_deref().is_some_and(looks_like_markdown)
    {
        eprintln!(
            "warning: body looks like Markdown but is being sent as storage; pass --representation markdown to convert it"
        );
    }

    match representation {
        BodyRepresentation::Markdown => body
            .map(|body| markdown_body_to_adf_with_options(body, markdown_options))
            .transpose()
            .map(|body| (body, ConfluenceBodyRepresentation::AtlasDocFormat)),
        _ if markdown_options.numbered_table_rows => {
            anyhow::bail!("--numbered-table-rows requires --representation markdown")
        }
        _ if !markdown_options.mentions.is_empty() => {
            anyhow::bail!("--mention requires --representation markdown")
        }
        _ => Ok((body, confluence_body_representation(representation)?)),
    }
}

pub(super) fn prepare_required_body_with_options(
    body: Option<String>,
    representation: BodyRepresentation,
    markdown_options: markdown::MarkdownToAdfOptions,
    missing_message: &str,
) -> anyhow::Result<(String, ConfluenceBodyRepresentation)> {
    let (body, representation) =
        prepare_optional_body_with_options(body, representation, markdown_options)?;
    Ok((
        body.ok_or_else(|| anyhow::anyhow!(missing_message.to_owned()))?,
        representation,
    ))
}

pub(super) fn markdown_body_to_adf_with_options(
    body: String,
    options: markdown::MarkdownToAdfOptions,
) -> anyhow::Result<String> {
    serde_json::to_string(&markdown::markdown_to_adf_with_options(&body, options))
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

pub(super) fn parse_view_fields(
    fields: Option<&str>,
    global: &GlobalArgs,
) -> anyhow::Result<Option<Vec<String>>> {
    let Some(fields) = fields else {
        return Ok(None);
    };
    if global.output != Some(OutputFormat::Json) {
        return Err(crate::error::UsageError("--fields requires --output json".to_owned()).into());
    }

    let mut parsed = Vec::new();
    for field in fields.split(',').map(str::trim) {
        if field.is_empty() {
            return Err(crate::error::UsageError(
                "--fields must contain non-empty comma-separated field names".to_owned(),
            )
            .into());
        }
        if !parsed.iter().any(|existing| existing == field) {
            parsed.push(field.to_owned());
        }
    }
    Ok(Some(parsed))
}

fn select_json_fields(
    value: serde_json::Value,
    fields: Option<&[String]>,
) -> anyhow::Result<serde_json::Value> {
    let Some(fields) = fields else {
        return Ok(value);
    };
    let object = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Confluence view output is not a JSON object"))?;
    let mut available = object.keys().cloned().collect::<Vec<_>>();
    available.push("schemaVersion".to_owned());
    available.sort();

    for field in fields {
        if field != "schemaVersion" && !object.contains_key(field) {
            return Err(crate::error::UsageError(format!(
                "unknown Confluence view field `{field}`; available fields: {}",
                available.join(", ")
            ))
            .into());
        }
    }

    let mut selected = serde_json::Map::new();
    if let Some(id) = object.get("id") {
        selected.insert("id".to_owned(), id.clone());
    }
    for field in fields {
        if field == "schemaVersion" || field == "id" {
            continue;
        }
        if let Some(value) = object.get(field) {
            selected.insert(field.clone(), value.clone());
        }
    }
    Ok(serde_json::Value::Object(selected))
}

struct BoundedBody {
    value: String,
    original_chars: usize,
    truncated: bool,
}

fn bound_body(body: String, max_chars: Option<usize>) -> BoundedBody {
    let original_chars = body.chars().count();
    let truncated = max_chars.is_some_and(|maximum| original_chars > maximum);
    let value = match max_chars.filter(|maximum| original_chars > *maximum) {
        Some(maximum) => body.chars().take(maximum).collect(),
        None => body,
    };
    BoundedBody {
        value,
        original_chars,
        truncated,
    }
}

fn content_view_format_name(format: ContentViewFormat) -> &'static str {
    match format {
        ContentViewFormat::Markdown => "markdown",
        ContentViewFormat::Storage => "storage",
        ContentViewFormat::AtlasDocFormat => "atlas_doc_format",
    }
}

fn body_view_command(profile_name: &str, resource: &str, id: &str) -> String {
    format!(
        "atla --profile {} --output json confluence {resource} view {} --format markdown",
        crate::pagination::quote(profile_name),
        crate::pagination::quote(id)
    )
}

fn warn_if_body_truncated(body: &BoundedBody, max_chars: Option<usize>) {
    if body.truncated {
        eprintln!(
            "warning: rendered body truncated from {} to {} characters; increase --max-chars to read more",
            body.original_chars,
            max_chars.unwrap_or(body.original_chars)
        );
    }
}

fn render_page_body(
    page: &ConfluencePage,
    format: ContentViewFormat,
    options: markdown::AdfToMarkdownOptions,
) -> anyhow::Result<String> {
    let body = page
        .body
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("page did not include a body"))?;
    if matches!(format, ContentViewFormat::Markdown) {
        Ok(serde_json::from_str::<serde_json::Value>(body)
            .map(|adf| markdown::adf_to_markdown_with_options(&adf, options))
            .unwrap_or_else(|_| body.to_owned()))
    } else {
        Ok(body.to_owned())
    }
}

pub(super) fn print_page_body_view(
    page: &ConfluencePage,
    format: ContentViewFormat,
    attachments: Option<&[ConfluenceAttachment]>,
    global: &GlobalArgs,
    options: markdown::AdfToMarkdownOptions,
    max_chars: Option<usize>,
    fields: Option<&[String]>,
) -> anyhow::Result<()> {
    let rendered = bound_body(render_page_body(page, format, options)?, max_chars);
    let format_name = content_view_format_name(format);
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => {
            let mut value = serde_json::to_value(page).context("failed to serialize page")?;
            if max_chars.is_some()
                && let Some(source) = page.body.as_deref()
            {
                value["body"] = serde_json::Value::Null;
                value["sourceBodyChars"] = source.chars().count().into();
                value["sourceBodyOmitted"] = true.into();
            }
            value["bodyIncluded"] = true.into();
            value["renderedBody"] = rendered.value.clone().into();
            value["renderedBodyChars"] = rendered.original_chars.into();
            value["renderedBodyTruncated"] = rendered.truncated.into();
            value["renderedFormat"] = format_name.into();
            if let Some(attachments) = attachments {
                value["attachments"] =
                    serde_json::to_value(attachments).context("failed to serialize attachments")?;
            }
            warn_if_body_truncated(&rendered, max_chars);
            output::print_json(&select_json_fields(value, fields)?)
        }
        OutputFormat::Csv => {
            println!("id,title,rendered_format,rendered_body,attachment_ids");
            let attachment_ids = attachments
                .unwrap_or_default()
                .iter()
                .filter_map(|attachment| attachment.id.as_deref())
                .collect::<Vec<_>>()
                .join(";");
            println!(
                "{},{},{},{},{}",
                output::csv_cell(page.id.as_deref().unwrap_or_default()),
                output::csv_cell(page.title.as_deref().unwrap_or_default()),
                output::csv_cell(format_name),
                output::csv_cell(&rendered.value),
                output::csv_cell(&attachment_ids),
            );
            warn_if_body_truncated(&rendered, max_chars);
            Ok(())
        }
        OutputFormat::Keys => {
            if let Some(id) = &page.id {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{}", rendered.value);
            warn_if_body_truncated(&rendered, max_chars);
            if let Some(attachments) = attachments {
                if attachments.is_empty() {
                    eprintln!("(no attachments)");
                } else {
                    print_attachments(attachments, global)?;
                }
            }
            Ok(())
        }
    }
}

pub(super) fn print_page_metadata_view(
    page: &ConfluencePage,
    id: &str,
    profile_name: &str,
    attachments: Option<&[ConfluenceAttachment]>,
    fields: Option<&[String]>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let command = body_view_command(profile_name, "page", id);
    if global.output == Some(OutputFormat::Json) {
        let mut value = serde_json::to_value(page).context("failed to serialize page")?;
        value["bodyIncluded"] = false.into();
        value["bodyCommand"] = command.clone().into();
        if let Some(attachments) = attachments {
            value["attachments"] =
                serde_json::to_value(attachments).context("failed to serialize attachments")?;
        }
        return output::print_json(&select_json_fields(value, fields)?);
    }

    if let Some(attachments) = attachments {
        print_page_with_attachments(page, attachments, global)?;
    } else {
        print_page(page, global)?;
    }
    eprintln!("Body omitted; run: {command}");
    Ok(())
}

pub(super) fn print_blog_body_view(
    post: &ConfluenceBlogPost,
    format: ContentViewFormat,
    global: &GlobalArgs,
    max_chars: Option<usize>,
    fields: Option<&[String]>,
) -> anyhow::Result<()> {
    let body = post
        .body
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("blog post did not include a body"))?;
    let rendered = if matches!(format, ContentViewFormat::Markdown) {
        serde_json::from_str::<serde_json::Value>(body)
            .map(|adf| {
                markdown::adf_to_markdown_with_options(
                    &adf,
                    markdown::AdfToMarkdownOptions::default(),
                )
            })
            .unwrap_or_else(|_| body.to_owned())
    } else {
        body.to_owned()
    };
    let rendered = bound_body(rendered, max_chars);
    let format_name = content_view_format_name(format);
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => {
            let mut value = serde_json::to_value(post).context("failed to serialize blog post")?;
            if max_chars.is_some() {
                value["body"] = serde_json::Value::Null;
                value["sourceBodyChars"] = body.chars().count().into();
                value["sourceBodyOmitted"] = true.into();
            }
            value["bodyIncluded"] = true.into();
            value["renderedBody"] = rendered.value.clone().into();
            value["renderedBodyChars"] = rendered.original_chars.into();
            value["renderedBodyTruncated"] = rendered.truncated.into();
            value["renderedFormat"] = format_name.into();
            warn_if_body_truncated(&rendered, max_chars);
            output::print_json(&select_json_fields(value, fields)?)
        }
        OutputFormat::Csv => {
            println!("id,title,rendered_format,rendered_body");
            println!(
                "{},{},{},{}",
                output::csv_cell(post.id.as_deref().unwrap_or_default()),
                output::csv_cell(post.title.as_deref().unwrap_or_default()),
                output::csv_cell(format_name),
                output::csv_cell(&rendered.value),
            );
            warn_if_body_truncated(&rendered, max_chars);
            Ok(())
        }
        OutputFormat::Keys => {
            if let Some(id) = &post.id {
                println!("{id}");
            }
            Ok(())
        }
        OutputFormat::Table => {
            println!("{}", rendered.value);
            warn_if_body_truncated(&rendered, max_chars);
            Ok(())
        }
    }
}

pub(super) fn print_blog_metadata_view(
    post: &ConfluenceBlogPost,
    id: &str,
    profile_name: &str,
    fields: Option<&[String]>,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let command = body_view_command(profile_name, "blog", id);
    if global.output == Some(OutputFormat::Json) {
        let mut value = serde_json::to_value(post).context("failed to serialize blog post")?;
        value["bodyIncluded"] = false.into();
        value["bodyCommand"] = command.clone().into();
        return output::print_json(&select_json_fields(value, fields)?);
    }

    print_blog_post(post, global)?;
    eprintln!("Body omitted; run: {command}");
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
    footer: Option<String>,
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
        footer,
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
    print_attachments_with_footer(attachments, global, None)
}

pub(super) fn print_attachments_with_footer(
    attachments: &[ConfluenceAttachment],
    global: &GlobalArgs,
    footer: Option<String>,
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
        footer,
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
    print_labels_with_footer(page, global, None)
}

pub(super) fn print_labels_with_footer(
    page: &ConfluenceLabelPage,
    global: &GlobalArgs,
    footer: Option<String>,
) -> anyhow::Result<()> {
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
        footer,
    )
}

pub(super) fn print_comments(
    page: &ConfluenceCommentPage,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    print_comments_with_footer(page, global, None)
}

pub(super) fn print_comments_with_footer(
    page: &ConfluenceCommentPage,
    global: &GlobalArgs,
    footer: Option<String>,
) -> anyhow::Result<()> {
    output::print_records(
        global.output.unwrap_or(OutputFormat::Table),
        page,
        page.results
            .iter()
            .filter_map(|comment| comment.id.clone())
            .collect(),
        &["id", "content_id", "status", "version", "body"],
        page.results
            .iter()
            .map(|comment| {
                vec![
                    comment.id.as_deref().unwrap_or("-").to_owned(),
                    comment
                        .page_id
                        .as_deref()
                        .or(comment.blog_post_id.as_deref())
                        .unwrap_or("-")
                        .to_owned(),
                    comment.status.as_deref().unwrap_or("-").to_owned(),
                    comment_version(comment).unwrap_or("-".to_owned()),
                    comment.body.as_deref().unwrap_or("-").replace('\n', " "),
                ]
            })
            .collect(),
        footer,
    )
}

pub(super) fn print_comment(
    comment: &ConfluenceComment,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    print_comments(
        &ConfluenceCommentPage {
            results: vec![comment.clone()],
            is_last: None,
            next_cursor: None,
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
    print_content_nodes_with_footer(nodes, global, None)
}

pub(super) fn print_content_nodes_with_footer(
    nodes: &[ConfluenceContentNode],
    global: &GlobalArgs,
    footer: Option<String>,
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
        footer,
    )
}

pub(super) fn print_pages(pages: &[ConfluencePage], global: &GlobalArgs) -> anyhow::Result<()> {
    print_pages_with_footer(pages, global, None)
}

pub(super) fn print_pages_with_footer(
    pages: &[ConfluencePage],
    global: &GlobalArgs,
    footer: Option<String>,
) -> anyhow::Result<()> {
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
        footer,
    )
}

/// Print a page alongside its attachments as a single unit. For JSON output, merges both into
/// one object with an `"attachments"` field to avoid emitting two separate JSON values.
pub(super) fn print_page_with_attachments(
    page: &ConfluencePage,
    attachments: &[ConfluenceAttachment],
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    match global.output.unwrap_or(OutputFormat::Table) {
        OutputFormat::Json => {
            let mut page_json = serde_json::to_value(page).context("failed to serialize page")?;
            page_json["attachments"] =
                serde_json::to_value(attachments).context("failed to serialize attachments")?;
            output::print_json(&page_json)
        }
        _ => {
            print_page(page, global)?;
            if attachments.is_empty() {
                eprintln!("(no attachments)");
            } else {
                print_attachments(attachments, global)?;
            }
            Ok(())
        }
    }
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
    print_blog_posts_with_footer(posts, global, None)
}

pub(super) fn print_blog_posts_with_footer(
    posts: &[ConfluenceBlogPost],
    global: &GlobalArgs,
    footer: Option<String>,
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
        footer,
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
    print_spaces_with_footer(spaces, global, None)
}

pub(super) fn print_spaces_with_footer(
    spaces: &[ConfluenceSpace],
    global: &GlobalArgs,
    footer: Option<String>,
) -> anyhow::Result<()> {
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
        footer,
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
            if let Some(space_owner_id) = &space.space_owner_id {
                println!("Space Owner ID: {space_owner_id}");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_warning_heuristic_is_conservative() {
        assert!(looks_like_markdown("# Runbook\n\n- recover"));
        assert!(looks_like_markdown("1. stop\n2. restore"));
        assert!(looks_like_markdown(
            "Read [the runbook](https://example.com)"
        ));
        assert!(!looks_like_markdown("<p>Storage body</p>"));
        assert!(!looks_like_markdown(
            "Plain text remains valid storage input."
        ));
    }

    #[test]
    fn converts_markdown_body_to_adf_write_body() {
        let (body, representation) = prepare_optional_body_with_options(
            Some("# Title\n\n**Important**".to_owned()),
            BodyRepresentation::Markdown,
            markdown::MarkdownToAdfOptions::default(),
        )
        .expect("convert markdown");

        assert_eq!(representation, ConfluenceBodyRepresentation::AtlasDocFormat);
        let body = body.expect("converted body");
        let adf: serde_json::Value = serde_json::from_str(&body).expect("adf json");
        assert_eq!(adf["type"], "doc");
        assert_eq!(adf["content"][0]["type"], "heading");
    }

    #[test]
    fn converts_markdown_body_to_adf_with_numbered_table_rows() {
        let (body, representation) = prepare_optional_body_with_options(
            Some("| Key | Value |\n| --- | --- |\n| Status | Done |".to_owned()),
            BodyRepresentation::Markdown,
            markdown::MarkdownToAdfOptions {
                numbered_table_rows: true,
                ..markdown::MarkdownToAdfOptions::default()
            },
        )
        .expect("convert markdown");

        assert_eq!(representation, ConfluenceBodyRepresentation::AtlasDocFormat);
        let body = body.expect("converted body");
        let adf: serde_json::Value = serde_json::from_str(&body).expect("adf json");
        assert_eq!(
            adf["content"][0]["attrs"]["isNumberColumnEnabled"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn rejects_numbered_table_rows_without_markdown_representation() {
        let error = prepare_optional_body_with_options(
            Some("<p>body</p>".to_owned()),
            BodyRepresentation::Storage,
            markdown::MarkdownToAdfOptions {
                numbered_table_rows: true,
                ..markdown::MarkdownToAdfOptions::default()
            },
        )
        .expect_err("numbered table rows should require markdown");

        assert!(
            error
                .to_string()
                .contains("requires --representation markdown")
        );
    }

    #[test]
    fn converts_markdown_body_to_adf_with_explicit_mention_mapping() {
        let (body, representation) = prepare_optional_body_with_options(
            Some("@Neo please review".to_owned()),
            BodyRepresentation::Markdown,
            markdown::MarkdownToAdfOptions {
                mentions: vec![markdown::MarkdownMention {
                    text: "Neo".to_owned(),
                    account_id: "account-neo".to_owned(),
                }],
                ..markdown::MarkdownToAdfOptions::default()
            },
        )
        .expect("convert markdown");

        assert_eq!(representation, ConfluenceBodyRepresentation::AtlasDocFormat);
        let body = body.expect("converted body");
        let adf: serde_json::Value = serde_json::from_str(&body).expect("adf json");
        assert_eq!(adf["content"][0]["content"][0]["type"], "mention");
        assert_eq!(
            adf["content"][0]["content"][0]["attrs"]["id"],
            "account-neo"
        );
    }

    #[test]
    fn parses_mention_mapping_argument() {
        let mention = parse_markdown_mention_arg("@Neo Hsu=abc-123").expect("parse mention");

        assert_eq!(mention.text, "Neo Hsu");
        assert_eq!(mention.account_id, "abc-123");
    }

    #[test]
    fn resolves_single_mention_user() {
        let resolution = resolve_mention_user(
            "Neo",
            vec![JiraUser {
                account_id: Some("account-neo".to_owned()),
                display_name: Some("Neo Hsu".to_owned()),
                active: Some(true),
            }],
        );

        assert_eq!(
            resolution,
            MentionUserResolution::Resolved(markdown::MarkdownMention {
                text: "Neo".to_owned(),
                account_id: "account-neo".to_owned(),
            })
        );
    }

    #[test]
    fn rejects_ambiguous_mention_users() {
        let resolution = resolve_mention_user(
            "Amy",
            vec![
                JiraUser {
                    account_id: Some("account-amy-1".to_owned()),
                    display_name: Some("Amy Chen".to_owned()),
                    active: Some(true),
                },
                JiraUser {
                    account_id: Some("account-amy-2".to_owned()),
                    display_name: Some("Amy Wang".to_owned()),
                    active: Some(true),
                },
            ],
        );

        assert!(matches!(resolution, MentionUserResolution::Ambiguous(_)));
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
