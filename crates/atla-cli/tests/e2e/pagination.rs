//! End-to-end tests for pagination behavior.
use super::support::*;

#[tokio::test]
async fn pagination_accumulates_across_pages() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .and(query_param("nextPageToken", "page-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [issue_json("PROJ-2")],
            "isLast": true,
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [issue_json("PROJ-1")],
            "isLast": false,
            "nextPageToken": "page-2",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "-o",
            "keys",
            "jira",
            "search",
            "project = PROJ",
            "--limit",
            "5",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "PROJ-1\nPROJ-2\n");
}

#[tokio::test]
async fn context_budgets_stop_pagination_and_preserve_resume_token() {
    for flag in ["--max-items", "--max-pages"] {
        let server = MockServer::start().await;
        let (_dir, config) = setup(&server.uri()).await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/search/jql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issues": [issue_json("PROJ-1")],
                "isLast": false,
                "nextPageToken": "page-2",
            })))
            .expect(1)
            .mount(&server)
            .await;

        let output = atla(
            &config,
            &[
                flag,
                "1",
                "-o",
                "json",
                "jira",
                "search",
                "project = PROJ",
                "--all",
            ],
        );

        assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
        let body: serde_json::Value =
            serde_json::from_str(&stdout(&output)).expect("JSON list output");
        assert_eq!(body["issues"].as_array().expect("issues").len(), 1);
        assert!(body["pagination"]["nextPageToken"].is_string());
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}

#[tokio::test]
async fn max_bytes_rejects_oversized_structured_output_without_partial_stdout() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [issue_json("PROJ-123456789")],
            "isLast": true,
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "--max-bytes",
            "40",
            "-o",
            "json",
            "jira",
            "search",
            "project = PROJ",
        ],
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty());
    let error: serde_json::Value =
        serde_json::from_str(&stderr(&output)).expect("structured budget error");
    assert_eq!(error["error"]["kind"], "usage");
    assert!(
        error["error"]["message"]
            .as_str()
            .expect("message")
            .contains("--max-bytes 40")
    );
    let non_json = atla(
        &config,
        &[
            "--max-bytes",
            "40",
            "--output",
            "keys",
            "jira",
            "search",
            "project = PROJ",
        ],
    );
    assert_eq!(non_json.status.code(), Some(2));
    assert!(stderr(&non_json).contains("requires --output json"));
    server.verify().await;
}
