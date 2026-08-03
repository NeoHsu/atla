//! End-to-end tests for plans behavior.
use super::support::*;

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
async fn oversized_saved_plan_is_rejected_before_write() {
    let server = MockServer::start().await;
    let (directory, config) = setup(&server.uri()).await;
    let body_path = directory.path().join("oversized-body.txt");
    let plan_path = directory.path().join("oversized-plan.json");
    std::fs::write(&body_path, vec![b'x'; 1024 * 1024 + 16 * 1024]).expect("write body");

    let output = atla(
        &config,
        &[
            "plan",
            "confluence",
            "page",
            "create",
            "--space-id",
            "123",
            "--title",
            "Oversized",
            "--body-file",
            body_path.to_str().expect("body path"),
            "--out",
            plan_path.to_str().expect("plan path"),
        ],
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(!plan_path.exists(), "oversized plan must not be written");
    let error: serde_json::Value =
        serde_json::from_str(&stderr(&output)).expect("structured usage error");
    assert_eq!(error["error"]["kind"], "usage");
    assert!(
        error["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("maximum is 1048576"))
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
