use super::*;

pub(in crate::commands::confluence) fn parse_view_fields(
    fields: Option<&str>,
    global: &Invocation,
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

pub(in crate::commands::confluence) fn select_json_fields(
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

pub(in crate::commands::confluence) struct BoundedBody {
    value: String,
    original_chars: usize,
    truncated: bool,
}

pub(in crate::commands::confluence) fn bound_body(
    body: String,
    max_chars: Option<usize>,
) -> BoundedBody {
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

pub(in crate::commands::confluence) fn content_view_format_name(
    format: ContentViewFormat,
) -> &'static str {
    match format {
        ContentViewFormat::Markdown => "markdown",
        ContentViewFormat::Storage => "storage",
        ContentViewFormat::AtlasDocFormat => "atlas_doc_format",
    }
}

pub(in crate::commands::confluence) fn body_view_command(
    profile_name: &str,
    resource: &str,
    id: &str,
) -> String {
    format!(
        "atla --profile {} --output json confluence {resource} view {} --format markdown",
        crate::pagination::quote(profile_name),
        crate::pagination::quote(id)
    )
}

pub(in crate::commands::confluence) fn warn_if_body_truncated(
    body: &BoundedBody,
    max_chars: Option<usize>,
) {
    if body.truncated {
        eprintln!(
            "warning: rendered body truncated from {} to {} characters; increase --max-chars to read more",
            body.original_chars,
            max_chars.unwrap_or(body.original_chars)
        );
    }
}

pub(in crate::commands::confluence) fn render_page_body(
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

pub(in crate::commands::confluence) fn print_page_body_view(
    page: &ConfluencePage,
    format: ContentViewFormat,
    attachments: Option<&[ConfluenceAttachment]>,
    global: &Invocation,
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
            global
                .output()
                .print_json(&select_json_fields(value, fields)?)
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

pub(in crate::commands::confluence) fn print_page_metadata_view(
    page: &ConfluencePage,
    id: &str,
    profile_name: &str,
    attachments: Option<&[ConfluenceAttachment]>,
    fields: Option<&[String]>,
    global: &Invocation,
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
        return global
            .output()
            .print_json(&select_json_fields(value, fields)?);
    }

    if let Some(attachments) = attachments {
        print_page_with_attachments(page, attachments, global)?;
    } else {
        print_page(page, global)?;
    }
    eprintln!("Body omitted; run: {command}");
    Ok(())
}

pub(in crate::commands::confluence) fn print_blog_body_view(
    post: &ConfluenceBlogPost,
    format: ContentViewFormat,
    global: &Invocation,
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
            global
                .output()
                .print_json(&select_json_fields(value, fields)?)
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

pub(in crate::commands::confluence) fn print_blog_metadata_view(
    post: &ConfluenceBlogPost,
    id: &str,
    profile_name: &str,
    fields: Option<&[String]>,
    global: &Invocation,
) -> anyhow::Result<()> {
    let command = body_view_command(profile_name, "blog", id);
    if global.output == Some(OutputFormat::Json) {
        let mut value = serde_json::to_value(post).context("failed to serialize blog post")?;
        value["bodyIncluded"] = false.into();
        value["bodyCommand"] = command.clone().into();
        return global
            .output()
            .print_json(&select_json_fields(value, fields)?);
    }

    print_blog_post(post, global)?;
    eprintln!("Body omitted; run: {command}");
    Ok(())
}
