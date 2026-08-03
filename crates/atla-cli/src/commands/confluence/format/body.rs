use super::*;

pub(in crate::commands::confluence) async fn resolve_space_id(
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

pub(in crate::commands::confluence) async fn resolve_required_space_id(
    client: &ConfluenceClient,
    space: Option<&str>,
    space_id: Option<String>,
) -> anyhow::Result<String> {
    resolve_space_id(client, space, space_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("provide --space or --space-id"))
}

pub(in crate::commands::confluence) fn read_body(
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

pub(in crate::commands::confluence) async fn markdown_to_adf_options_for_body(
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

pub(in crate::commands::confluence) fn parse_markdown_mention_arg(
    raw: &str,
) -> anyhow::Result<markdown::MarkdownMention> {
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

pub(in crate::commands::confluence) fn mention_matches(
    mention: &markdown::MarkdownMention,
    candidate: &str,
) -> bool {
    mention.text.trim().eq_ignore_ascii_case(candidate.trim())
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::commands::confluence) enum MentionUserResolution {
    Resolved(markdown::MarkdownMention),
    NotFound,
    Ambiguous(Vec<String>),
}

pub(in crate::commands::confluence) fn resolve_mention_user(
    query: &str,
    users: Vec<JiraUser>,
) -> MentionUserResolution {
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

pub(in crate::commands::confluence) fn user_to_mention(
    query: &str,
    user: &JiraUser,
) -> markdown::MarkdownMention {
    markdown::MarkdownMention {
        text: query.trim().trim_start_matches('@').to_owned(),
        account_id: user
            .account_id
            .as_deref()
            .expect("filtered users include account ids")
            .to_owned(),
    }
}

pub(in crate::commands::confluence) fn looks_like_markdown(body: &str) -> bool {
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

pub(in crate::commands::confluence) fn prepare_optional_body_with_options(
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

pub(in crate::commands::confluence) fn prepare_required_body_with_options(
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

pub(in crate::commands::confluence) fn markdown_body_to_adf_with_options(
    body: String,
    options: markdown::MarkdownToAdfOptions,
) -> anyhow::Result<String> {
    serde_json::to_string(&markdown::markdown_to_adf_with_options(&body, options))
        .context("failed to encode Markdown body as Atlas Doc Format")
}

pub(in crate::commands::confluence) fn confluence_body_representation(
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

pub(in crate::commands::confluence) fn status_from_draft(draft: bool) -> ConfluenceContentStatus {
    if draft {
        ConfluenceContentStatus::Draft
    } else {
        ConfluenceContentStatus::Current
    }
}

pub(in crate::commands::confluence) fn view_format_body_representation(
    format: ContentViewFormat,
) -> Option<ConfluenceBodyRepresentation> {
    match format {
        ContentViewFormat::Markdown | ContentViewFormat::AtlasDocFormat => {
            Some(ConfluenceBodyRepresentation::AtlasDocFormat)
        }
        ContentViewFormat::Storage => Some(ConfluenceBodyRepresentation::Storage),
    }
}
