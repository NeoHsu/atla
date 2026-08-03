use super::*;

pub(in crate::commands::jira) fn read_optional_text(
    value: Option<String>,
    file: Option<&Path>,
) -> anyhow::Result<Option<String>> {
    if let Some(file) = file {
        return std::fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))
            .map(Some);
    }

    Ok(value.map(|s| unescape_text(&s)))
}

pub(in crate::commands::jira) fn unescape_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek().copied() {
                Some('n') => {
                    chars.next();
                    result.push('\n');
                }
                Some('t') => {
                    chars.next();
                    result.push('\t');
                }
                Some('r') => {
                    chars.next();
                    result.push('\r');
                }
                Some('\\') => {
                    chars.next();
                    result.push('\\');
                }
                _ => {
                    result.push(c);
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub(in crate::commands::jira) fn read_required_text(
    value: Option<String>,
    file: Option<&Path>,
    name: &str,
) -> anyhow::Result<String> {
    read_optional_text(value, file)?.ok_or_else(|| anyhow::anyhow!("missing {name}"))
}

pub(in crate::commands::jira) fn parse_label_update(
    issue_id_or_key: &str,
    labels: Option<&str>,
) -> anyhow::Result<JiraIssueLabelUpdate> {
    let mut update = JiraIssueLabelUpdate {
        issue_id_or_key: issue_id_or_key.to_owned(),
        add: Vec::new(),
        remove: Vec::new(),
    };
    let Some(labels) = labels else {
        return Ok(update);
    };

    let parts: Vec<&str> = labels
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();

    // Detect whether the user is using add:/remove: prefix syntax.
    let uses_prefix = parts
        .iter()
        .any(|p| p.starts_with("add:") || p.starts_with("remove:"));

    for part in parts {
        if uses_prefix {
            let (action, label) = part.split_once(':').ok_or_else(|| {
                anyhow::anyhow!(
                    "mixed label syntax: when using add:/remove: prefixes all parts must use them, got `{part}`"
                )
            })?;
            if label.is_empty() {
                anyhow::bail!("label cannot be empty in `{part}`");
            }
            match action {
                "add" => update.add.push(label.to_owned()),
                "remove" => update.remove.push(label.to_owned()),
                _ => anyhow::bail!("unsupported label operation `{action}`; use add or remove"),
            }
        } else {
            // Plain name style: all parts are additions.
            update.add.push(part.to_owned());
        }
    }

    Ok(update)
}

pub(in crate::commands::jira) fn parse_fields(
    fields: &[String],
) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
    let mut parsed = serde_json::Map::new();
    for field in fields {
        let (name, raw) = field
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected --field name=value, got `{field}`"))?;
        if name.is_empty() {
            anyhow::bail!("field name cannot be empty in `{field}`");
        }
        let value = parse_field_value(name, raw)?;
        parsed.insert(name.to_owned(), value);
    }

    Ok(parsed)
}

/// Parse a field value: try JSON first; if that fails and the value is a plain
/// identifier (no JSON structural chars), auto-wrap it as `{"name": value}`.
///
/// Note: plain string fields (e.g. text custom fields) must be quoted as JSON strings:
/// `--field myfield="some value"` sends the string directly without wrapping.
pub(in crate::commands::jira) fn parse_field_value(
    name: &str,
    raw: &str,
) -> anyhow::Result<serde_json::Value> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) {
        return Ok(v);
    }
    // Plain identifier (no braces, brackets, colons, or quotes): wrap as {"name": raw}
    let looks_like_plain = !raw.starts_with('{')
        && !raw.starts_with('[')
        && !raw.starts_with('"')
        && !raw.contains(':');
    if looks_like_plain {
        return Ok(match name {
            "assignee" => serde_json::json!({ "accountId": raw }),
            "parent" => serde_json::json!({ "key": raw }),
            _ => serde_json::json!({ "name": raw }),
        });
    }
    anyhow::bail!(
        "invalid value for field `{name}`: `{raw}`\n  \
         Tip: use JSON for objects/arrays (e.g. --field '{name}={{\"name\":\"High\"}}'),\n  \
         a plain name (e.g. --field '{name}=High') for option fields,\n  \
         or a quoted JSON string (e.g. --field '{name}=\"some text\"') for text fields"
    )
}

pub(in crate::commands::jira) fn parse_issue_fields(
    fields: Option<&str>,
) -> anyhow::Result<Option<Vec<String>>> {
    let Some(fields) = fields else {
        return Ok(None);
    };
    let parsed = fields
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if parsed.is_empty() {
        anyhow::bail!("--fields must include at least one field");
    }

    Ok(Some(parsed))
}

pub(in crate::commands::jira) fn issue_fields_for_url(fields: Option<&[String]>) -> String {
    fields
        .filter(|fields| !fields.is_empty())
        .map(|fields| fields.join(","))
        .unwrap_or_else(|| "summary,status,assignee,issuetype,priority".to_owned())
}

pub(in crate::commands::jira) fn display_extra_issue_fields(
    fields: Option<&[String]>,
) -> Vec<String> {
    let Some(fields) = fields else {
        return Vec::new();
    };
    if fields.iter().any(|field| field == "*all") {
        return Vec::new();
    }
    fields
        .iter()
        .filter(|field| {
            !matches!(
                field.as_str(),
                "summary" | "status" | "assignee" | "issuetype" | "priority"
            )
        })
        .cloned()
        .collect()
}

pub(in crate::commands::jira) fn issue_field_cell(issue: &JiraIssue, field: &str) -> String {
    issue
        .fields
        .get(field)
        .map(value_cell)
        .unwrap_or_else(|| "-".to_owned())
}

pub(in crate::commands::jira) fn value_cell(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "-".to_owned(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Array(values) => {
            values.iter().map(value_cell).collect::<Vec<_>>().join(", ")
        }
        serde_json::Value::Object(object) => object
            .get("name")
            .or_else(|| object.get("displayName"))
            .or_else(|| object.get("value"))
            .or_else(|| object.get("key"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| serde_json::to_string(value).unwrap_or_else(|_| "-".to_owned())),
    }
}

pub(in crate::commands::jira) fn select_transition(
    transitions: &[JiraTransition],
) -> anyhow::Result<&JiraTransition> {
    let items = transitions
        .iter()
        .map(transition_display)
        .collect::<Vec<_>>();
    let index = Select::new()
        .with_prompt("Transition")
        .items(&items)
        .default(0)
        .interact()
        .context("failed to read transition selection")?;

    transitions
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("selected transition was out of range"))
}

pub(in crate::commands::jira) fn transition_display(transition: &JiraTransition) -> String {
    let name = transition.name.as_deref().unwrap_or("-");
    let to_status = transition
        .to_status
        .as_ref()
        .and_then(|status| status.name.as_deref());
    match to_status {
        Some(status) if status != name => format!("{name} → {status}"),
        _ => name.to_owned(),
    }
}

pub(in crate::commands::jira) fn can_prompt(global: &Invocation) -> bool {
    !global.no_input && stdin().is_terminal() && stdout().is_terminal()
}

pub(in crate::commands::jira) fn open_web_url(url: &str) -> anyhow::Result<()> {
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
