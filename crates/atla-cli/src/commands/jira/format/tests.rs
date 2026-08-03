use super::*;

#[test]
fn parses_issue_fields_list() {
    let fields = parse_issue_fields(Some("summary, description,attachment"))
        .expect("issue fields")
        .expect("some fields");

    assert_eq!(
        fields,
        vec![
            "summary".to_owned(),
            "description".to_owned(),
            "attachment".to_owned()
        ]
    );
}

#[test]
fn parses_transition_field_json_values() {
    let fields = parse_fields(&[
        r#"customfield_12345={"value":"Ready"}"#.to_owned(),
        r#"customfield_67890="2026-05-18""#.to_owned(),
    ])
    .expect("transition fields");

    assert_eq!(
        fields["customfield_12345"],
        serde_json::json!({ "value": "Ready" })
    );
    assert_eq!(fields["customfield_67890"], serde_json::json!("2026-05-18"));
}

#[test]
fn requested_all_fields_does_not_expand_table_columns() {
    let fields = vec!["*all".to_owned()];

    assert!(display_extra_issue_fields(Some(&fields)).is_empty());
}
