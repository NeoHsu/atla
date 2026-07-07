//! End-to-end tests: the real `atla` binary against a mock Atlassian API.
//! Covers the agent-facing contract: exit codes, error bodies, structured
//! JSON errors, output formats, pagination, and dry-run.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup(server_uri: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        format!(
            r#"
[default]
profile = "e2e"

[profiles.e2e]
instance = "{server_uri}"
email = "e2e@example.com"
"#
        ),
    )
    .expect("write config");
    (dir, config_path)
}

fn atla(config: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_atla"))
        .env("ATLA_CONFIG", config)
        .env("ATLA_TOKEN", "test-token")
        .env_remove("ATLA_API_TOKEN")
        .env_remove("ATLA_CREDENTIALS")
        .args(args)
        .output()
        .expect("run atla")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn issue_json(key: &str) -> serde_json::Value {
    serde_json::json!({
        "id": "10001",
        "key": key,
        "fields": {
            "summary": format!("Summary for {key}"),
            "status": {"name": "To Do"},
        }
    })
}

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

#[tokio::test]
async fn missing_profile_exits_auth_code() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    let output = atla(
        &config,
        &["--profile", "ghost", "jira", "issue", "view", "PROJ-1"],
    );
    assert_eq!(output.status.code(), Some(3));
    assert!(stderr(&output).contains("atla auth login --profile ghost"));
}

#[tokio::test]
async fn dry_run_makes_no_request() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;
    // No mocks mounted: any request would 404 and change the output.

    let output = atla(
        &config,
        &["--dry-run", "jira", "issue", "delete", "PROJ-1", "--yes"],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(
        stdout(&output).contains("Would DELETE"),
        "dry-run should describe the request, got: {}",
        stdout(&output)
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn dry_run_create_prints_request_body() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    let output = atla(
        &config,
        &[
            "--dry-run",
            "jira",
            "issue",
            "create",
            "--project",
            "PROJ",
            "--type",
            "Bug",
            "--summary",
            "Crash on save",
            "--field",
            "customfield_10166=\"5.1.0\"",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("Would POST"), "got: {out}");
    let body_json = out.split_once("Request body:").expect("body printed").1;
    let body: serde_json::Value = serde_json::from_str(body_json.trim()).expect("body is JSON");
    assert_eq!(body["fields"]["summary"], "Crash on save");
    assert_eq!(body["fields"]["customfield_10166"], "5.1.0");
    assert_eq!(body["fields"]["project"]["key"], "PROJ");
    assert!(server.received_requests().await.unwrap().is_empty());
}

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
