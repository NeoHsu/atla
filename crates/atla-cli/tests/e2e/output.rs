//! End-to-end tests for output behavior.
use super::support::*;

#[tokio::test]
async fn search_success_json_includes_issue_keys() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [issue_json("PROJ-1")],
            "isLast": true,
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(&config, &["-o", "json", "jira", "search", "project = PROJ"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("stdout is JSON");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["issues"][0]["key"], "PROJ-1");
}

#[tokio::test]
async fn api_error_body_reaches_stderr() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "errorMessages": ["Error in the JQL Query: Expecting operator but got 'oops'."],
        })))
        .mount(&server)
        .await;

    let output = atla(&config, &["jira", "search", "project = oops"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("Expecting operator but got 'oops'"),
        "API error body must surface, got: {}",
        stderr(&output)
    );
}

#[tokio::test]
async fn exit_codes_classify_auth_not_found_retryable() {
    for (status, expected_exit) in [(401, 3), (403, 3), (404, 4), (429, 5), (503, 5)] {
        let server = MockServer::start().await;
        let (_dir, config) = setup(&server.uri()).await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/PROJ-1"))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(serde_json::json!({"errorMessages": ["nope"]})),
            )
            .mount(&server)
            .await;

        let output = atla(&config, &["jira", "issue", "view", "PROJ-1"]);
        assert_eq!(
            output.status.code(),
            Some(expected_exit),
            "HTTP {status} should exit {expected_exit}, stderr: {}",
            stderr(&output)
        );
    }
}

#[tokio::test]
async fn json_output_emits_structured_error() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/PROJ-404"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({"errorMessages": ["Issue does not exist"]})),
        )
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &["-o", "json", "jira", "issue", "view", "PROJ-404"],
    );
    assert_eq!(output.status.code(), Some(4));
    let error: serde_json::Value =
        serde_json::from_str(&stderr(&output)).expect("stderr is a JSON error object");
    assert_eq!(error["schemaVersion"], 1);
    assert_eq!(error["error"]["kind"], "not_found");
    assert_eq!(error["error"]["status"], 404);
    assert_eq!(error["error"]["retryable"], false);
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Issue does not exist")
    );
}
