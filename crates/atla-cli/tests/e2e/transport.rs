//! End-to-end tests for transport behavior.
use super::support::*;

#[tokio::test]
async fn transient_429_is_retried_on_raw_endpoints() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    // First response is a 429 with a fast Retry-After; the retry succeeds.
    Mock::given(method("GET"))
        .and(path("/rest/agile/1.0/board"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "1"))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/agile/1.0/board"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "values": [{"id": 84, "name": "Platform board", "type": "scrum"}],
            "isLast": true,
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(&config, &["jira", "board", "list"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Platform board"));
}

#[tokio::test]
async fn generated_get_clients_retry_transient_statuses() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/PROJ-1"))
        .respond_with(ResponseTemplate::new(503).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/PROJ-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(issue_json("PROJ-1")))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(&config, &["-o", "json", "jira", "issue", "view", "PROJ-1"]);

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let issue: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("issue JSON");
    assert_eq!(issue["key"], "PROJ-1");
}

#[tokio::test]
async fn generated_put_failures_remain_retryable_after_bounded_retries() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    Mock::given(method("PUT"))
        .and(path("/rest/api/3/issue/PROJ-1"))
        .respond_with(
            ResponseTemplate::new(503)
                .insert_header("Retry-After", "0")
                .set_body_string("temporarily unavailable"),
        )
        .expect(3)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "--output",
            "json",
            "jira",
            "issue",
            "update",
            "PROJ-1",
            "--summary",
            "Retry safely",
        ],
    );

    assert_eq!(output.status.code(), Some(5), "stderr: {}", stderr(&output));
    let error: serde_json::Value =
        serde_json::from_str(&stderr(&output)).expect("structured PUT error");
    assert_eq!(error["error"]["kind"], "retryable");
    assert_eq!(error["error"]["retryable"], true);
    server.verify().await;
}

#[tokio::test]
async fn generated_post_failures_are_non_retryable_and_marked_ambiguous() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .respond_with(ResponseTemplate::new(503).set_body_string("temporarily unavailable"))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "-o",
            "json",
            "jira",
            "issue",
            "create",
            "--project",
            "PROJ",
            "--type",
            "Task",
            "--summary",
            "Ambiguous create",
        ],
    );

    assert_eq!(output.status.code(), Some(1));
    let error: serde_json::Value =
        serde_json::from_str(&stderr(&output)).expect("structured mutation error");
    assert_eq!(error["error"]["kind"], "ambiguous_mutation");
    assert_eq!(error["error"]["retryable"], false);
    server.verify().await;
}

#[tokio::test]
async fn transient_503_is_not_retried_for_post_mutations() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("POST"))
        .and(path("/rest/agile/1.0/sprint"))
        .respond_with(ResponseTemplate::new(503).set_body_string("temporarily unavailable"))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "jira", "sprint", "create", "--board", "84", "--name", "Release",
        ],
    );

    assert_eq!(output.status.code(), Some(1), "stderr: {}", stderr(&output));
    assert!(stderr(&output).contains("outcome is unknown"));
    server.verify().await;
}

#[tokio::test]
async fn cli_timeout_bounds_slow_mutation_requests() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("POST"))
        .and(path("/rest/agile/1.0/sprint"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(2))
                .set_body_json(serde_json::json!({"id": 1, "name": "Release"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "--timeout",
            "1",
            "jira",
            "sprint",
            "create",
            "--board",
            "84",
            "--name",
            "Release",
        ],
    );

    assert_eq!(output.status.code(), Some(1), "stderr: {}", stderr(&output));
    assert!(stderr(&output).contains("outcome is unknown"));
    server.verify().await;
}

#[tokio::test]
async fn binary_attachment_download_preserves_bytes() {
    let server = MockServer::start().await;
    let (dir, config) = setup(&server.uri()).await;
    let destination = dir.path().join("artifact.bin");
    let content_url = format!("{}/download/123", server.uri());

    Mock::given(method("GET"))
        .and(path("/rest/api/3/attachment/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "123",
            "filename": "artifact.bin",
            "mimeType": "application/octet-stream",
            "size": 6,
            "content": content_url,
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/download/123"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0, 159, 146, 150, 255, 10]))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "jira",
            "issue",
            "attachment",
            "download",
            "123",
            "--dest",
            destination.to_str().expect("UTF-8 temp path"),
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert_eq!(
        std::fs::read(&destination).expect("downloaded file"),
        vec![0, 159, 146, 150, 255, 10]
    );
    server.verify().await;
}
