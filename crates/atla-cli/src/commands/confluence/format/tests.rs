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
