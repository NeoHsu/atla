//! End-to-end tests for formats behavior.
use super::support::*;

#[tokio::test]
async fn output_formats_render_the_same_data() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [issue_json("PROJ-7")],
            "isLast": true,
        })))
        .mount(&server)
        .await;

    let keys = atla(&config, &["-o", "keys", "jira", "search", "x = y"]);
    assert_eq!(stdout(&keys), "PROJ-7\n");

    let csv = atla(&config, &["-o", "csv", "jira", "search", "x = y"]);
    let csv_out = stdout(&csv);
    let mut lines = csv_out.lines();
    let header = lines.next().expect("csv header");
    assert!(
        header.to_lowercase().starts_with("key"),
        "csv header should start with key column, got: {header}"
    );
    assert!(lines.next().expect("csv row").contains("PROJ-7"));

    let table = atla(&config, &["jira", "search", "x = y"]);
    assert!(stdout(&table).contains("PROJ-7"));
}
