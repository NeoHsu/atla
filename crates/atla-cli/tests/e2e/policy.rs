//! End-to-end tests for policy behavior.
use super::support::*;

#[tokio::test]
async fn explain_policy_reports_deny_allow_mode_and_global_read_only() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;
    let mut contents = std::fs::read_to_string(&config).expect("read config");
    contents.push_str(
        r#"
[profiles.e2e.policy]
mode = "read-only"
allow = ["jira.issue.*"]
deny = ["jira.issue.delete"]
"#,
    );
    std::fs::write(&config, contents).expect("write policy config");

    let allowed = atla(
        &config,
        &["explain-policy", "jira.issue.create", "--output", "json"],
    );
    assert!(allowed.status.success(), "{}", stderr(&allowed));
    let allowed_json: serde_json::Value =
        serde_json::from_slice(&allowed.stdout).expect("allowed policy JSON");
    assert_eq!(allowed_json["allowed"], true);
    assert_eq!(allowed_json["profileDecision"], "allow-rule");
    assert_eq!(allowed_json["matchedPattern"], "jira.issue.*");

    let denied = atla(
        &config,
        &["explain-policy", "jira.issue.delete", "--output", "json"],
    );
    assert!(denied.status.success(), "{}", stderr(&denied));
    let denied_json: serde_json::Value =
        serde_json::from_slice(&denied.stdout).expect("denied policy JSON");
    assert_eq!(denied_json["allowed"], false);
    assert_eq!(denied_json["profileDecision"], "deny-rule");

    let read_only = atla(
        &config,
        &[
            "--read-only",
            "explain-policy",
            "jira.issue.create",
            "--output",
            "json",
        ],
    );
    assert!(read_only.status.success(), "{}", stderr(&read_only));
    let read_only_json: serde_json::Value =
        serde_json::from_slice(&read_only.stdout).expect("read-only policy JSON");
    assert_eq!(read_only_json["allowed"], false);
    assert_eq!(read_only_json["globalReadOnlyBlocked"], true);

    let local = atla(
        &config,
        &["explain-policy", "auth.login", "--output", "json"],
    );
    assert!(local.status.success(), "{}", stderr(&local));
    let local_json: serde_json::Value =
        serde_json::from_slice(&local.stdout).expect("local policy JSON");
    assert_eq!(local_json["allowed"], true);
    assert_eq!(local_json["profileDecision"], "not-applicable");

    let no_profile_config = _dir.path().join("no-profile.toml");
    std::fs::write(&no_profile_config, "schema_version = 2\n").expect("write empty config");
    let no_profile = atla(
        &no_profile_config,
        &["explain-policy", "jira.issue.create", "--output", "json"],
    );
    assert!(no_profile.status.success(), "{}", stderr(&no_profile));
    let no_profile_json: serde_json::Value =
        serde_json::from_slice(&no_profile.stdout).expect("no-profile policy JSON");
    assert_eq!(no_profile_json["profileFound"], false);
    assert_eq!(no_profile_json["allowed"], false);
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
