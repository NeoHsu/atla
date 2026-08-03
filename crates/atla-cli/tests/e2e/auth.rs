//! End-to-end tests for auth behavior.
use super::support::*;

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

#[test]
fn auth_status_without_a_profile_remains_machine_readable() {
    let directory = tempfile::tempdir().expect("tempdir");
    let config = directory.path().join("missing-config.toml");

    let output = atla_with_stdin(&config, &["--output", "json", "auth", "status"], b"");

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());
    let status: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("auth status JSON");
    assert_eq!(status["schemaVersion"], 1);
    assert_eq!(status["configured"], false);
    assert_eq!(status["profile"], serde_json::Value::Null);
    assert_eq!(status["token"], "missing");
}

#[tokio::test]
async fn auth_status_with_a_profile_reports_configuration_and_token_source() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;

    let output = atla(&config, &["--output", "json", "auth", "status"]);

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let status: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("auth status JSON");
    assert_eq!(status["schemaVersion"], 1);
    assert_eq!(status["configured"], true);
    assert_eq!(status["profile"], "e2e");
    assert_eq!(status["token"], "provided by environment");
    assert!(server.received_requests().await.unwrap().is_empty());
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
async fn local_discovery_commands_emit_versioned_json_without_network() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;

    let doctor = atla(&config, &["doctor", "--output", "json"]);
    assert!(doctor.status.success(), "{}", stderr(&doctor));
    let doctor_json: serde_json::Value =
        serde_json::from_slice(&doctor.stdout).expect("doctor JSON");
    assert_eq!(doctor_json["schemaVersion"], 1);
    assert_eq!(doctor_json["healthy"], true);
    assert!(doctor_json["checks"].as_array().is_some_and(|checks| {
        checks
            .iter()
            .any(|check| check["name"] == "site-reachability" && check["status"] == "skipped")
    }));

    let operations = atla(&config, &["operation", "list", "--output", "json"]);
    assert!(operations.status.success(), "{}", stderr(&operations));
    let operations_json: serde_json::Value =
        serde_json::from_slice(&operations.stdout).expect("operation JSON");
    assert_eq!(operations_json["schemaVersion"], 1);
    assert_eq!(operations_json["pagination"]["isLast"], true);
    assert!(
        operations_json["operations"]
            .as_array()
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item["id"] == "jira.issue.create"
                        && item["risk"] == "write"
                        && item["mutating"] == true
                })
            })
    );

    let schemas = atla(&config, &["schema", "list", "--output", "json"]);
    assert!(schemas.status.success(), "{}", stderr(&schemas));
    let schemas_json: serde_json::Value =
        serde_json::from_slice(&schemas.stdout).expect("schema list JSON");
    assert!(
        schemas_json["schemas"]
            .as_array()
            .is_some_and(|items| { items.iter().any(|item| item["name"] == "operation-list-v1") })
    );

    let schema = atla(
        &config,
        &["schema", "print", "error-v1", "--output", "json"],
    );
    assert!(schema.status.success(), "{}", stderr(&schema));
    let schema_json: serde_json::Value =
        serde_json::from_slice(&schema.stdout).expect("printed schema JSON");
    assert_eq!(
        schema_json["$id"],
        "https://github.com/NeoHsu/atla/docs/schemas/error-v1.schema.json"
    );
    assert!(schema_json.get("schemaVersion").is_none());

    assert!(
        server
            .received_requests()
            .await
            .expect("received requests")
            .is_empty()
    );
}

#[tokio::test]
async fn doctor_accepts_matching_skill_version_without_network() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;
    let output = atla(
        &config,
        &[
            "doctor",
            "--skill-version",
            env!("CARGO_PKG_VERSION"),
            "--output",
            "json",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("doctor compatibility JSON");
    assert_eq!(report["cliVersion"], env!("CARGO_PKG_VERSION"));
    assert_eq!(report["skillCompatibility"]["compatible"], true);
    assert_eq!(report["skillCompatibility"]["recommendedAction"], "none");
    assert!(report["skillCompatibility"]["updateCommand"].is_null());
    assert!(report["checks"].as_array().is_some_and(|checks| {
        checks
            .iter()
            .any(|check| check["name"] == "skill-version" && check["status"] == "ok")
    }));
    assert!(
        server
            .received_requests()
            .await
            .expect("received requests")
            .is_empty()
    );
}

#[tokio::test]
async fn doctor_fails_closed_with_actionable_version_mismatch() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;
    std::fs::write(&config, "this is not valid TOML = [").expect("write invalid config");
    let version_output = atla(&config, &["--version"]);
    assert_eq!(
        version_output.status.code(),
        Some(0),
        "{}",
        stderr(&version_output)
    );
    assert!(stdout(&version_output).contains(env!("CARGO_PKG_VERSION")));

    let current_skill_tag = format!("/tree/v{}", env!("CARGO_PKG_VERSION"));
    for (skill_version, action, command_fragment) in [
        ("0.0.0", "update-skill", current_skill_tag),
        ("999.0.0", "update-cli", "--tag v999.0.0".to_owned()),
    ] {
        let output = atla(
            &config,
            &[
                "doctor",
                "--skill-version",
                skill_version,
                "--output",
                "json",
                "--network",
            ],
        );

        assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("mismatch doctor JSON");
        assert_eq!(report["healthy"], false);
        assert_eq!(report["skillCompatibility"]["compatible"], false);
        assert_eq!(report["skillCompatibility"]["recommendedAction"], action);
        assert!(
            report["skillCompatibility"]["updateCommand"]
                .as_str()
                .is_some_and(|command| command.contains(&command_fragment))
        );
        assert_eq!(report["checks"].as_array().map(Vec::len), Some(1));
        assert_eq!(report["checks"][0]["name"], "skill-version");

        let error: serde_json::Value =
            serde_json::from_str(stderr(&output).trim()).expect("mismatch error JSON");
        assert_eq!(error["error"]["kind"], "version_mismatch");
        assert_eq!(error["error"]["retryable"], false);
    }

    assert!(
        server
            .received_requests()
            .await
            .expect("received requests")
            .is_empty()
    );
}

#[tokio::test]
async fn doctor_network_check_discovers_and_validates_cloud_id() {
    let server = MockServer::start().await;
    let (_dir, config) = setup(&server.uri()).await;
    Mock::given(method("GET"))
        .and(path("/_edge/tenant_info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cloudId": "cloud-doctor"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &["doctor", "--network", "--output", "json", "--timeout", "2"],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("doctor network JSON");
    assert!(value["checks"].as_array().is_some_and(|checks| {
        checks.iter().any(|check| {
            check["name"] == "site-reachability"
                && check["status"] == "ok"
                && check["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("cloud-doctor"))
        })
    }));
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
