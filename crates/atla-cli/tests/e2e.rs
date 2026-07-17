//! End-to-end tests: the real `atla` binary against a mock Atlassian API.
//! Covers the agent-facing contract: exit codes, error bodies, structured
//! JSON errors, output formats, pagination, and dry-run.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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
        .env(
            "ATLA_CREDENTIALS",
            config.with_file_name("credentials.toml"),
        )
        .args(args)
        .output()
        .expect("run atla")
}

fn atla_with_stdin(config: &Path, args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_atla"))
        .env("ATLA_CONFIG", config)
        .env_remove("ATLA_TOKEN")
        .env_remove("ATLA_API_TOKEN")
        .env(
            "ATLA_CREDENTIALS",
            config.with_file_name("credentials.toml"),
        )
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn atla");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(input)
        .expect("write stdin");
    child.wait_with_output().expect("wait for atla")
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
async fn auth_discover_prints_cloud_id_and_product_endpoints() {
    let server = MockServer::start().await;
    let server_uri = server.uri();
    let (_directory, config) = setup(&server_uri).await;
    Mock::given(method("GET"))
        .and(path("/_edge/tenant_info"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"cloudId": "cloud-123"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &["-o", "json", "auth", "discover", "--site", &server_uri],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let discovery: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("discovery JSON");
    assert_eq!(discovery["cloudId"], "cloud-123");
    assert_eq!(
        discovery["jiraEndpoint"],
        "https://api.atlassian.com/ex/jira/cloud-123"
    );
}

#[tokio::test]
async fn auth_login_reads_token_from_stdin_without_echoing_it() {
    let server = MockServer::start().await;
    let server_uri = server.uri();
    let (directory, config) = setup(&server_uri).await;

    let output = atla_with_stdin(
        &config,
        &[
            "--no-input",
            "--profile",
            "stdin",
            "auth",
            "login",
            "--instance",
            &server_uri,
            "--email",
            "stdin@example.com",
            "--storage",
            "file",
            "--token-stdin",
        ],
        b"stdin-secret-token\n",
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(!stdout(&output).contains("stdin-secret-token"));
    assert!(!stderr(&output).contains("stdin-secret-token"));
    let credentials = std::fs::read_to_string(directory.path().join("credentials.toml"))
        .expect("file credential store");
    assert!(credentials.contains("stdin-secret-token"));
    assert!(server.received_requests().await.unwrap().is_empty());
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
    let unsupported_json = atla(
        &config,
        &[
            "--output",
            "json",
            "--dry-run",
            "jira",
            "issue",
            "delete",
            "PROJ-1",
            "--yes",
        ],
    );
    assert_eq!(unsupported_json.status.code(), Some(2));
    assert!(stdout(&unsupported_json).is_empty());
    let error: serde_json::Value =
        serde_json::from_str(&stderr(&unsupported_json)).expect("JSON dry-run error");
    assert_eq!(error["error"]["kind"], "usage");

    let blocked_label = atla(
        &config,
        &[
            "--output",
            "json",
            "confluence",
            "page",
            "label",
            "remove",
            "123",
            "obsolete",
        ],
    );
    assert_eq!(blocked_label.status.code(), Some(2));
    assert!(stderr(&blocked_label).contains("requires --yes"));
    let preview_label = atla(
        &config,
        &[
            "--dry-run",
            "confluence",
            "page",
            "label",
            "remove",
            "123",
            "obsolete",
        ],
    );
    assert_eq!(
        preview_label.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&preview_label)
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn destructive_guard_runs_before_config_migration_or_network() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    let original = std::fs::read(&config).expect("read original config");

    let output = atla(
        &config,
        &[
            "--output",
            "json",
            "confluence",
            "page",
            "label",
            "remove",
            "123",
            "obsolete",
        ],
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("requires --yes"));
    assert_eq!(std::fs::read(&config).expect("read config"), original);
    assert!(!config.with_file_name("config.toml.v1.bak").exists());
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn read_only_policy_blocks_mutations_before_network_access() {
    let server = MockServer::start().await;
    let (directory, config) = setup(&server.uri()).await;
    let original_config = std::fs::read(&config).expect("read original config");
    let plan_path = directory.path().join("blocked-plan.json");
    let plan_path_arg = plan_path.to_string_lossy().into_owned();

    let blocked = atla(
        &config,
        &[
            "--read-only",
            "-o",
            "json",
            "jira",
            "issue",
            "delete",
            "PROJ-1",
            "--yes",
        ],
    );
    let allowed = atla(
        &config,
        &[
            "--read-only",
            "--dry-run",
            "jira",
            "issue",
            "view",
            "PROJ-1",
        ],
    );
    let mutation_preview = atla(
        &config,
        &[
            "--read-only",
            "--dry-run",
            "jira",
            "issue",
            "delete",
            "PROJ-1",
            "--yes",
        ],
    );
    let blocked_plan = atla(
        &config,
        &[
            "--read-only",
            "plan",
            "jira",
            "issue",
            "create",
            "--project",
            "PROJ",
            "--type",
            "Task",
            "--summary",
            "Blocked plan",
            "--out",
            &plan_path_arg,
        ],
    );

    assert_eq!(blocked.status.code(), Some(2));
    let error: serde_json::Value =
        serde_json::from_str(&stderr(&blocked)).expect("structured policy error");
    assert_eq!(error["error"]["kind"], "usage");
    assert!(
        error["error"]["message"]
            .as_str()
            .expect("message")
            .contains("jira.issue.delete")
    );
    assert_eq!(
        allowed.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&allowed)
    );
    assert_eq!(
        mutation_preview.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&mutation_preview)
    );
    assert!(stdout(&mutation_preview).contains("Would DELETE"));
    assert_eq!(blocked_plan.status.code(), Some(2));
    assert!(!plan_path.exists());
    assert!(server.received_requests().await.unwrap().is_empty());
    assert_eq!(
        std::fs::read(&config).expect("read config"),
        original_config
    );
    assert!(!config.with_file_name("config.toml.v1.bak").exists());
}

#[tokio::test]
async fn profile_policy_enforces_mode_allow_and_deny_before_network() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    let mut contents = std::fs::read_to_string(&config).expect("read config");
    contents.push_str(
        r#"
[profiles.e2e.policy]
mode = "read-only"
allow = ["jira.sprint.create", "jira.issue.delete"]
deny = ["*.delete"]
"#,
    );
    std::fs::write(&config, contents).expect("write policy");

    Mock::given(method("POST"))
        .and(path("/rest/agile/1.0/sprint"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 1, "name": "Allowed"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let blocked_by_mode = atla(
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
            "Blocked",
        ],
    );
    let blocked_by_deny = atla(
        &config,
        &["-o", "json", "jira", "issue", "delete", "PROJ-1", "--yes"],
    );
    let allowed = atla(
        &config,
        &[
            "jira", "sprint", "create", "--board", "84", "--name", "Allowed",
        ],
    );

    assert_eq!(blocked_by_mode.status.code(), Some(2));
    assert_eq!(blocked_by_deny.status.code(), Some(2));
    assert!(stderr(&blocked_by_deny).contains("blocked by policy"));
    assert_eq!(
        allowed.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&allowed)
    );
    server.verify().await;
}

#[tokio::test]
async fn scoped_profile_dry_runs_use_product_gateway_urls() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;
    let mut contents = std::fs::read_to_string(&config).expect("read config");
    contents.push_str("cloud_id = \"cloud-123\"\n");
    std::fs::write(&config, contents).expect("write scoped profile");

    let jira = atla(
        &config,
        &["--dry-run", "jira", "issue", "delete", "PROJ-1", "--yes"],
    );
    let confluence = atla(
        &config,
        &["--dry-run", "confluence", "page", "delete", "456", "--yes"],
    );

    assert_eq!(jira.status.code(), Some(0), "stderr: {}", stderr(&jira));
    assert!(
        stdout(&jira)
            .contains("https://api.atlassian.com/ex/jira/cloud-123/rest/api/3/issue/PROJ-1")
    );
    assert_eq!(
        confluence.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&confluence)
    );
    assert!(
        stdout(&confluence)
            .contains("https://api.atlassian.com/ex/confluence/cloud-123/wiki/api/v2/pages/456")
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn saved_plan_applies_only_after_hash_and_policy_validation() {
    let server = MockServer::start().await;
    let (directory, config) = setup(&server.uri()).await;
    let plan_path = directory.path().join("create-plan.json");
    let plan_path_arg = plan_path.to_string_lossy().into_owned();
    let description_path = directory.path().join("description.txt");
    let description_path_arg = description_path.to_string_lossy().into_owned();
    std::fs::write(&description_path, "Original description").expect("write description");

    let collision = atla(
        &config,
        &[
            "plan",
            "jira",
            "issue",
            "create",
            "--project",
            "PROJ",
            "--type",
            "Task",
            "--summary",
            "Collision",
            "--description-file",
            &description_path_arg,
            "--out",
            &description_path_arg,
        ],
    );
    assert_eq!(collision.status.code(), Some(2));
    assert_eq!(
        std::fs::read_to_string(&description_path).expect("read preserved description"),
        "Original description"
    );

    let planned = atla(
        &config,
        &[
            "plan",
            "jira",
            "issue",
            "create",
            "--project",
            "PROJ",
            "--type",
            "Task",
            "--summary",
            "Saved plan",
            "--description-file",
            &description_path_arg,
            "--out",
            &plan_path_arg,
        ],
    );
    assert_eq!(
        planned.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&planned)
    );
    let summary: serde_json::Value =
        serde_json::from_str(&stdout(&planned)).expect("plan summary JSON");
    assert_eq!(summary["operation"], "jira.issue.create");
    assert!(
        summary["planHash"]
            .as_str()
            .expect("plan hash")
            .starts_with("sha256:")
    );
    let plan: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&plan_path).expect("read plan")).expect("plan JSON");
    assert_eq!(plan["operation"], "jira.issue.create");
    assert_eq!(plan["inputFiles"].as_array().expect("input files").len(), 1);
    assert!(
        plan["inputFiles"][0]["sha256"]
            .as_str()
            .expect("input hash")
            .starts_with("sha256:")
    );
    assert_eq!(
        plan["requests"][0]["body"]["fields"]["summary"],
        "Saved plan"
    );
    assert!(
        !std::fs::read_to_string(&plan_path)
            .expect("plan text")
            .contains("test-token")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&plan_path)
                .expect("plan metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert!(server.received_requests().await.unwrap().is_empty());

    let unconfirmed = atla(&config, &["apply", &plan_path_arg]);
    assert_eq!(unconfirmed.status.code(), Some(2));
    assert!(stderr(&unconfirmed).contains("requires --yes"));

    let mut config_text = std::fs::read_to_string(&config).expect("read migrated config");
    config_text.push_str("\n[profiles.e2e.policy]\nmode = \"read-only\"\n");
    std::fs::write(&config, config_text).expect("write read-only profile policy");
    let blocked = atla(
        &config,
        &["--output", "json", "apply", &plan_path_arg, "--yes"],
    );
    assert_eq!(blocked.status.code(), Some(2));
    assert!(stderr(&blocked).contains("blocked by policy"));
    let policy_update = atla(
        &config,
        &[
            "config",
            "set",
            "profiles.e2e.policy.allow",
            "jira.issue.create",
        ],
    );
    assert_eq!(
        policy_update.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&policy_update)
    );

    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "10001",
            "key": "PROJ-1"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let applied = atla(
        &config,
        &["--output", "json", "apply", &plan_path_arg, "--yes"],
    );
    assert_eq!(
        applied.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&applied)
    );
    let receipt: serde_json::Value =
        serde_json::from_str(&stdout(&applied)).expect("apply receipt JSON");
    assert_eq!(receipt["operation"], "jira.issue.create");
    assert_eq!(receipt["target"], "PROJ-1");

    std::fs::write(&description_path, "Changed description").expect("change description");
    let changed_input = atla(
        &config,
        &["--output", "json", "apply", &plan_path_arg, "--yes"],
    );
    assert_eq!(changed_input.status.code(), Some(2));
    assert!(stderr(&changed_input).contains("input file hash mismatch"));
    std::fs::write(&description_path, "Original description").expect("restore description");

    let mut tampered = plan;
    tampered["requests"][0]["body"]["fields"]["summary"] = "Tampered".into();
    std::fs::write(
        &plan_path,
        serde_json::to_vec_pretty(&tampered).expect("encode tampered plan"),
    )
    .expect("write tampered plan");
    let rejected = atla(
        &config,
        &["--output", "json", "apply", &plan_path_arg, "--yes"],
    );
    assert_eq!(rejected.status.code(), Some(2));
    assert!(stderr(&rejected).contains("plan hash mismatch"));
    server.verify().await;
}

#[tokio::test]
async fn saved_confluence_plan_applies_converted_markdown_body() {
    let server = MockServer::start().await;
    let (directory, config) = setup(&server.uri()).await;
    let plan_path = directory.path().join("page-plan.json");
    let plan_path_arg = plan_path.to_string_lossy().into_owned();

    let planned = atla(
        &config,
        &[
            "plan",
            "confluence",
            "page",
            "create",
            "--space-id",
            "123",
            "--title",
            "Runbook",
            "--body",
            "# Hello",
            "--representation",
            "markdown",
            "--out",
            &plan_path_arg,
        ],
    );
    assert_eq!(
        planned.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&planned)
    );
    let plan: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&plan_path).expect("read page plan"))
            .expect("page plan JSON");
    assert_eq!(plan["operation"], "confluence.page.create");
    assert_eq!(
        plan["requests"][0]["body"]["body"]["representation"],
        "atlas_doc_format"
    );
    assert!(server.received_requests().await.unwrap().is_empty());

    Mock::given(method("POST"))
        .and(path("/wiki/api/v2/pages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": "456", "title": "Runbook"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let applied = atla(
        &config,
        &["--output", "json", "apply", &plan_path_arg, "--yes"],
    );
    assert_eq!(
        applied.status.code(),
        Some(0),
        "stderr: {}",
        stderr(&applied)
    );
    let receipt: serde_json::Value =
        serde_json::from_str(&stdout(&applied)).expect("page receipt JSON");
    assert_eq!(receipt["operation"], "confluence.page.create");
    assert_eq!(receipt["target"], "456");
    server.verify().await;
}

#[tokio::test]
async fn json_mutation_success_includes_receipt_metadata() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    Mock::given(method("POST"))
        .and(path("/rest/agile/1.0/sprint"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id": 321, "name": "Receipt"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "--output", "json", "jira", "sprint", "create", "--board", "84", "--name", "Receipt",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let receipt: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("receipt JSON");
    assert_eq!(receipt["schemaVersion"], 1);
    assert_eq!(receipt["operation"], "jira.sprint.create");
    assert_eq!(receipt["profile"], "e2e");
    assert_eq!(receipt["target"], 321);
    assert_eq!(receipt["requestId"], serde_json::Value::Null);
    assert!(receipt["completedAt"].as_str().is_some());
    server.verify().await;
}

#[tokio::test]
async fn json_dry_run_emits_versioned_operation_plan() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;

    let output = atla(
        &config,
        &[
            "--output",
            "json",
            "--dry-run",
            "jira",
            "issue",
            "create",
            "--project",
            "PROJ",
            "--type",
            "Task",
            "--summary",
            "Planned",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("plan JSON");
    assert_eq!(plan["schemaVersion"], 1);
    assert_eq!(plan["planVersion"], 1);
    assert_eq!(plan["operation"], "jira.issue.create");
    assert_eq!(plan["mutating"], true);
    assert_eq!(plan["requests"][0]["method"], "POST");
    assert_eq!(plan["requests"][0]["body"]["fields"]["summary"], "Planned");
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
async fn confluence_page_dry_run_prints_converted_request_body_without_network() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    let output = atla(
        &config,
        &[
            "--dry-run",
            "confluence",
            "page",
            "create",
            "--space-id",
            "123",
            "--title",
            "Runbook",
            "--body",
            "# Recovery",
            "--representation",
            "markdown",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    let body_json = out.split_once("Request body:").expect("body printed").1;
    let body: serde_json::Value = serde_json::from_str(body_json.trim()).expect("body is JSON");
    assert_eq!(body["spaceId"], "123");
    assert_eq!(body["title"], "Runbook");
    assert_eq!(body["body"]["representation"], "atlas_doc_format");
    let adf: serde_json::Value = serde_json::from_str(
        body["body"]["value"]
            .as_str()
            .expect("ADF body should be encoded as a string"),
    )
    .expect("body value is ADF JSON");
    assert_eq!(adf["type"], "doc");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn confluence_page_update_dry_run_prints_exact_body_without_network() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    let output = atla(
        &config,
        &[
            "--dry-run",
            "confluence",
            "page",
            "update",
            "456",
            "--title",
            "Updated runbook",
            "--body",
            "<p>Ready</p>",
            "--representation",
            "storage",
            "--version",
            "8",
            "--message",
            "agent preview",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    let body_json = out.split_once("Request body:").expect("body printed").1;
    let body: serde_json::Value = serde_json::from_str(body_json.trim()).expect("body is JSON");
    assert_eq!(body["id"], "456");
    assert_eq!(body["title"], "Updated runbook");
    assert_eq!(body["body"]["value"], "<p>Ready</p>");
    assert_eq!(body["version"]["number"], 8);
    assert_eq!(body["version"]["message"], "agent preview");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn confluence_body_views_emit_one_json_document_with_supplementary_data() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/111"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "111",
            "status": "current",
            "title": "Runbook",
            "spaceId": "123",
            "body": {"storage": {"value": "<p>Hello</p>"}}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/111/attachments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": []
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/blogposts/222"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "222",
            "status": "current",
            "title": "Release",
            "spaceId": "123",
            "body": {"storage": {"value": "<p>Shipped</p>"}}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let page = atla(
        &config,
        &[
            "--output",
            "json",
            "confluence",
            "page",
            "view",
            "111",
            "--format",
            "storage",
            "--with-attachments",
        ],
    );
    let blog = atla(
        &config,
        &[
            "--output",
            "json",
            "confluence",
            "blog",
            "view",
            "222",
            "--format",
            "storage",
        ],
    );

    assert_eq!(page.status.code(), Some(0), "stderr: {}", stderr(&page));
    let page_json: serde_json::Value =
        serde_json::from_str(&stdout(&page)).expect("single page JSON document");
    assert_eq!(page_json["renderedBody"], "<p>Hello</p>");
    assert_eq!(page_json["renderedFormat"], "storage");
    assert_eq!(page_json["attachments"], serde_json::json!([]));
    assert_eq!(blog.status.code(), Some(0), "stderr: {}", stderr(&blog));
    let blog_json: serde_json::Value =
        serde_json::from_str(&stdout(&blog)).expect("single blog JSON document");
    assert_eq!(blog_json["renderedBody"], "<p>Shipped</p>");
    assert_eq!(blog_json["renderedFormat"], "storage");
    server.verify().await;
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
        .respond_with(ResponseTemplate::new(503))
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
        .respond_with(ResponseTemplate::new(503).set_body_string("temporarily unavailable"))
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
