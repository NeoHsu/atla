use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PageToken {
    JiraJql {
        next_page_token: String,
        query_hash: String,
    },
    ConfluencePageCursor {
        cursor: String,
        query_hash: String,
    },
    ConfluenceCursor {
        resource: String,
        cursor: String,
        query_hash: String,
    },
    ConfluenceOffset {
        resource: String,
        start: u32,
        query_hash: String,
    },
    JiraOffset {
        resource: String,
        start_at: u64,
        query_hash: String,
    },
}

impl PageToken {
    pub fn encode(&self) -> anyhow::Result<String> {
        let json = serde_json::to_vec(self)?;
        Ok(URL_SAFE_NO_PAD.encode(json))
    }

    pub fn decode(value: &str) -> anyhow::Result<Self> {
        let bytes = URL_SAFE_NO_PAD
            .decode(value)
            .map_err(|e| anyhow::anyhow!("invalid --page-token: {e}"))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| anyhow::anyhow!("invalid --page-token payload: {e}"))
    }
}

pub fn jira_jql_query_hash(jql: &str, fields: Option<&[String]>) -> String {
    let mut normalized_fields = fields.unwrap_or(&[]).to_vec();
    normalized_fields.sort();
    stable_hash(
        &serde_json::json!({
            "kind": "jira.jql",
            "jql": jql,
            "fields": normalized_fields,
        })
        .to_string(),
    )
}

pub fn jira_jql_next_token(
    next_page_token: Option<String>,
    jql: &str,
    fields: Option<&[String]>,
) -> anyhow::Result<Option<String>> {
    let Some(next_page_token) = next_page_token else {
        return Ok(None);
    };
    PageToken::JiraJql {
        next_page_token,
        query_hash: jira_jql_query_hash(jql, fields),
    }
    .encode()
    .map(Some)
}

pub fn decode_jira_jql_token(
    page_token: Option<&str>,
    jql: &str,
    fields: Option<&[String]>,
) -> anyhow::Result<Option<String>> {
    let Some(page_token) = page_token else {
        return Ok(None);
    };
    match PageToken::decode(page_token)? {
        PageToken::JiraJql {
            next_page_token,
            query_hash,
        } => {
            let expected = jira_jql_query_hash(jql, fields);
            if query_hash != expected {
                anyhow::bail!(
                    "--page-token was generated for a different Jira JQL query or field set"
                );
            }
            Ok(Some(next_page_token))
        }
        PageToken::ConfluencePageCursor { .. }
        | PageToken::ConfluenceCursor { .. }
        | PageToken::ConfluenceOffset { .. }
        | PageToken::JiraOffset { .. } => {
            anyhow::bail!("--page-token was generated for a different command, not Jira JQL")
        }
    }
}

pub fn confluence_page_query_hash(space_id: Option<&str>, title: Option<&str>) -> String {
    stable_hash(
        &serde_json::json!({
            "kind": "confluence.page.list",
            "spaceId": space_id,
            "title": title,
        })
        .to_string(),
    )
}

pub fn confluence_page_next_token(
    cursor: Option<String>,
    space_id: Option<&str>,
    title: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    PageToken::ConfluencePageCursor {
        cursor,
        query_hash: confluence_page_query_hash(space_id, title),
    }
    .encode()
    .map(Some)
}

pub fn decode_confluence_page_token(
    page_token: Option<&str>,
    space_id: Option<&str>,
    title: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let Some(page_token) = page_token else {
        return Ok(None);
    };
    match PageToken::decode(page_token)? {
        PageToken::ConfluencePageCursor { cursor, query_hash } => {
            let expected = confluence_page_query_hash(space_id, title);
            if query_hash != expected {
                anyhow::bail!("--page-token was generated for a different Confluence page query");
            }
            Ok(Some(cursor))
        }
        PageToken::JiraJql { .. }
        | PageToken::ConfluenceCursor { .. }
        | PageToken::ConfluenceOffset { .. }
        | PageToken::JiraOffset { .. } => {
            anyhow::bail!(
                "--page-token was generated for a different command, not Confluence pages"
            )
        }
    }
}

pub fn confluence_page_next_command(
    space_id: Option<&str>,
    title: Option<&str>,
    limit: u32,
    token: &str,
) -> String {
    let mut parts = vec![
        "atla".to_owned(),
        "confluence".to_owned(),
        "page".to_owned(),
        "list".to_owned(),
        "--limit".to_owned(),
        limit.to_string(),
    ];
    if let Some(space_id) = space_id {
        parts.push("--space-id".to_owned());
        parts.push(shell_quote(space_id));
    }
    if let Some(title) = title {
        parts.push("--title".to_owned());
        parts.push(shell_quote(title));
    }
    parts.push("--page-token".to_owned());
    parts.push(shell_quote(token));
    parts.join(" ")
}

pub fn jira_search_next_command(
    jql: &str,
    limit: u32,
    fields: Option<&[String]>,
    token: &str,
) -> String {
    let mut parts = vec![
        "atla".to_owned(),
        "jira".to_owned(),
        "search".to_owned(),
        shell_quote(jql),
        "--limit".to_owned(),
        limit.to_string(),
    ];
    if let Some(fields) = fields.filter(|fields| !fields.is_empty()) {
        parts.push("--fields".to_owned());
        parts.push(shell_quote(&fields.join(",")));
    }
    parts.push("--page-token".to_owned());
    parts.push(shell_quote(token));
    parts.join(" ")
}

pub fn query_hash(resource: &str, parts: &[(&str, String)]) -> String {
    let mut parts = parts.to_vec();
    parts.sort_by(|a, b| a.0.cmp(b.0));
    stable_hash(&serde_json::json!({ "resource": resource, "parts": parts }).to_string())
}

pub fn confluence_cursor_next_token(
    resource: &str,
    cursor: Option<String>,
    query_hash: String,
) -> anyhow::Result<Option<String>> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    PageToken::ConfluenceCursor {
        resource: resource.to_owned(),
        cursor,
        query_hash,
    }
    .encode()
    .map(Some)
}

pub fn decode_confluence_cursor_token(
    page_token: Option<&str>,
    resource: &str,
    query_hash: String,
) -> anyhow::Result<Option<String>> {
    let Some(page_token) = page_token else {
        return Ok(None);
    };
    match PageToken::decode(page_token)? {
        PageToken::ConfluenceCursor {
            resource: token_resource,
            cursor,
            query_hash: token_hash,
        } if token_resource == resource && token_hash == query_hash => Ok(Some(cursor)),
        PageToken::ConfluenceCursor {
            resource: token_resource,
            ..
        } => {
            anyhow::bail!("--page-token was generated for a different query ({token_resource})")
        }
        _ => anyhow::bail!("--page-token was generated for a different command"),
    }
}

pub fn confluence_offset_next_token(
    resource: &str,
    start: Option<u32>,
    query_hash: String,
) -> anyhow::Result<Option<String>> {
    let Some(start) = start else {
        return Ok(None);
    };
    PageToken::ConfluenceOffset {
        resource: resource.to_owned(),
        start,
        query_hash,
    }
    .encode()
    .map(Some)
}

pub fn decode_confluence_offset_token(
    page_token: Option<&str>,
    resource: &str,
    query_hash: String,
) -> anyhow::Result<u32> {
    let Some(page_token) = page_token else {
        return Ok(0);
    };
    match PageToken::decode(page_token)? {
        PageToken::ConfluenceOffset {
            resource: token_resource,
            start,
            query_hash: token_hash,
        } if token_resource == resource && token_hash == query_hash => Ok(start),
        PageToken::ConfluenceOffset {
            resource: token_resource,
            ..
        } => {
            anyhow::bail!("--page-token was generated for a different query ({token_resource})")
        }
        _ => anyhow::bail!("--page-token was generated for a different command"),
    }
}

pub fn jira_offset_next_token(
    resource: &str,
    start_at: Option<u64>,
    query_hash: String,
) -> anyhow::Result<Option<String>> {
    let Some(start_at) = start_at else {
        return Ok(None);
    };
    PageToken::JiraOffset {
        resource: resource.to_owned(),
        start_at,
        query_hash,
    }
    .encode()
    .map(Some)
}

pub fn decode_jira_offset_token(
    page_token: Option<&str>,
    resource: &str,
    query_hash: String,
) -> anyhow::Result<u64> {
    let Some(page_token) = page_token else {
        return Ok(0);
    };
    match PageToken::decode(page_token)? {
        PageToken::JiraOffset {
            resource: token_resource,
            start_at,
            query_hash: token_hash,
        } if token_resource == resource && token_hash == query_hash => Ok(start_at),
        PageToken::JiraOffset {
            resource: token_resource,
            ..
        } => {
            anyhow::bail!("--page-token was generated for a different query ({token_resource})")
        }
        _ => anyhow::bail!("--page-token was generated for a different command"),
    }
}

pub fn next_command(mut parts: Vec<String>, limit: u32, token: &str) -> String {
    parts.push("--limit".to_owned());
    parts.push(limit.to_string());
    parts.push("--page-token".to_owned());
    parts.push(shell_quote(token));
    parts.join(" ")
}

pub fn quote(value: &str) -> String {
    shell_quote(value)
}

pub fn next_page_footer(next_command: &str) -> String {
    format!("More results available.\nNext page:\n  {next_command}")
}

fn stable_hash(value: &str) -> String {
    // FNV-1a: sufficient as a compact, deterministic guard against using a
    // page token with a different command/query. This is not a security hash.
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn shell_quote(value: &str) -> String {
    shell_words::quote(value).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jira_jql_token_round_trips_and_validates_query() {
        let fields = vec!["summary".to_owned(), "status".to_owned()];
        let token = jira_jql_next_token(
            Some("server-token".to_owned()),
            "project = ABC",
            Some(&fields),
        )
        .expect("encode")
        .expect("token");

        let decoded =
            decode_jira_jql_token(Some(&token), "project = ABC", Some(&fields)).expect("decode");
        assert_eq!(decoded.as_deref(), Some("server-token"));

        let err = decode_jira_jql_token(Some(&token), "project = XYZ", Some(&fields))
            .expect_err("wrong query should fail");
        assert!(err.to_string().contains("different Jira JQL query"));
    }

    #[test]
    fn confluence_cursor_token_round_trips_and_validates_resource() {
        let hash = query_hash("confluence.space.list", &[("key", "ENG".to_owned())]);
        let token = confluence_cursor_next_token(
            "confluence.space.list",
            Some("cursor-1".to_owned()),
            hash.clone(),
        )
        .expect("encode")
        .expect("token");

        let decoded =
            decode_confluence_cursor_token(Some(&token), "confluence.space.list", hash.clone())
                .expect("decode");
        assert_eq!(decoded.as_deref(), Some("cursor-1"));

        let err = decode_confluence_cursor_token(Some(&token), "confluence.blog.list", hash)
            .expect_err("wrong resource should fail");
        assert!(err.to_string().contains("different query"));
    }

    #[test]
    fn offset_tokens_round_trip() {
        let hash = query_hash("jira.project.list", &[("query", "abc".to_owned())]);
        let token = jira_offset_next_token("jira.project.list", Some(25), hash.clone())
            .expect("encode")
            .expect("token");
        assert_eq!(
            decode_jira_offset_token(Some(&token), "jira.project.list", hash).expect("decode"),
            25
        );

        let hash = query_hash("confluence.search", &[("cql", "type=page".to_owned())]);
        let token = confluence_offset_next_token("confluence.search", Some(50), hash.clone())
            .expect("encode")
            .expect("token");
        assert_eq!(
            decode_confluence_offset_token(Some(&token), "confluence.search", hash)
                .expect("decode"),
            50
        );
    }

    #[test]
    fn next_command_quotes_arguments_and_appends_limit_and_token() {
        let command = next_command(
            vec![
                "atla".to_owned(),
                "jira".to_owned(),
                "search".to_owned(),
                quote("project = ABC ORDER BY updated DESC"),
            ],
            10,
            "abc.def",
        );

        assert!(command.contains("--limit 10"));
        assert!(command.contains("--page-token abc.def"));
        assert!(command.contains("'project = ABC ORDER BY updated DESC'"));
    }
}
