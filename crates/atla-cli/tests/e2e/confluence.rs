//! End-to-end tests for confluence behavior.
use super::support::*;

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
async fn confluence_metadata_view_is_self_describing_and_projects_json_fields() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/111"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "111",
            "status": "current",
            "title": "Runbook",
            "spaceId": "123"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "--output",
            "json",
            "confluence",
            "page",
            "view",
            "111",
            "--metadata-only",
            "--fields",
            "title,bodyIncluded,bodyCommand",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let value: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("metadata view JSON");
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["id"], "111");
    assert_eq!(value["title"], "Runbook");
    assert_eq!(value["bodyIncluded"], false);
    assert_eq!(
        value["bodyCommand"],
        "atla --profile e2e --output json confluence page view 111 --format markdown"
    );
    assert!(value.get("status").is_none());
    server.verify().await;
}

#[tokio::test]
async fn confluence_fields_require_json_before_network_access() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;

    let output = atla(
        &config,
        &[
            "confluence",
            "page",
            "view",
            "111",
            "--metadata-only",
            "--fields",
            "title",
        ],
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("--fields requires --output json"));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn confluence_body_view_truncates_unicode_safely_and_projects_fields() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/blogposts/222"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "222",
            "status": "current",
            "title": "Release",
            "spaceId": "123",
            "body": {"storage": {"value": "部署完成abc"}}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
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
            "--max-chars",
            "3",
            "--fields",
            "title,body,bodyIncluded,sourceBodyChars,sourceBodyOmitted,renderedBody,renderedBodyChars,renderedBodyTruncated,renderedFormat",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let value: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("bounded body JSON");
    assert_eq!(value["id"], "222");
    assert_eq!(value["renderedBody"], "部署完");
    assert_eq!(value["renderedBodyChars"], 7);
    assert_eq!(value["renderedBodyTruncated"], true);
    assert_eq!(value["bodyIncluded"], true);
    assert!(value["body"].is_null());
    assert_eq!(value["sourceBodyChars"], 7);
    assert_eq!(value["sourceBodyOmitted"], true);
    assert!(stderr(&output).contains("truncated from 7 to 3 characters"));
    server.verify().await;
}

#[tokio::test]
async fn likely_markdown_storage_input_emits_actionable_warning_without_network() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;

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
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(stderr(&output).contains("pass --representation markdown"));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn confluence_space_json_exposes_space_owner_id() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "ENG"))
        .and(query_param("limit", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "id": "123",
                "key": "ENG",
                "name": "Engineering",
                "type": "global",
                "status": "current",
                "spaceOwnerId": "owner-123"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &["--output", "json", "confluence", "space", "view", "ENG"],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("space JSON");
    assert_eq!(value["spaceOwnerId"], "owner-123");
    server.verify().await;
}

#[tokio::test]
async fn confluence_blog_markdown_dry_run_converts_to_adf_without_network() {
    let server = MockServer::start().await;
    let (directory, config) = setup(&server.uri()).await;
    let body = directory.path().join("blog.md");
    std::fs::write(&body, "# Release\n\nReady to ship.\n").expect("write Markdown body");

    let output = atla(
        &config,
        &[
            "--output",
            "json",
            "--dry-run",
            "confluence",
            "blog",
            "create",
            "--space-id",
            "123",
            "--title",
            "Release",
            "--body-file",
            body.to_str().expect("UTF-8 temp path"),
            "--representation",
            "markdown",
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let plan: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("structured blog plan");
    assert_eq!(plan["operation"], "confluence.blog.create");
    assert_eq!(
        plan["requests"][0]["body"]["body"]["representation"],
        "atlas_doc_format"
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn confluence_attachment_upload_json_is_a_versioned_object() {
    let server = MockServer::start().await;
    let (directory, config) = setup(&server.uri()).await;
    let file = directory.path().join("evidence.txt");
    std::fs::write(&file, b"evidence\n").expect("write attachment");
    Mock::given(method("PUT"))
        .and(path("/wiki/rest/api/content/123/child/attachment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "id": "att456",
                "type": "attachment",
                "status": "current",
                "title": "evidence.txt",
                "version": {"number": 1},
                "_links": {"download": "/download/attachments/123/evidence.txt"}
            }],
            "size": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = atla(
        &config,
        &[
            "--output",
            "json",
            "confluence",
            "attachment",
            "upload",
            "123",
            file.to_str().expect("UTF-8 temp path"),
        ],
    );

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let uploaded: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("versioned attachment result");
    assert_eq!(uploaded["schemaVersion"], 1);
    assert_eq!(uploaded["operation"], "confluence.attachment.upload");
    assert_eq!(uploaded["results"][0]["id"], "att456");
    server.verify().await;
}

#[tokio::test]
async fn confluence_delete_options_are_omitted_unless_enabled() {
    let server = MockServer::start().await;
    let (_directory, config) = setup(&server.uri()).await;
    for endpoint in [
        "/wiki/api/v2/pages/123",
        "/wiki/api/v2/blogposts/123",
        "/wiki/api/v2/attachments/123",
    ] {
        Mock::given(method("DELETE"))
            .and(path(endpoint))
            .respond_with(ResponseTemplate::new(204))
            .expect(2)
            .mount(&server)
            .await;
    }

    for arguments in [
        vec!["confluence", "page", "delete", "123", "--yes"],
        vec!["confluence", "page", "delete", "123", "--purge", "--yes"],
        vec!["confluence", "blog", "delete", "123", "--yes"],
        vec!["confluence", "blog", "delete", "123", "--purge", "--yes"],
        vec!["confluence", "attachment", "delete", "att123", "--yes"],
        vec![
            "confluence",
            "attachment",
            "delete",
            "att123",
            "--purge",
            "--yes",
        ],
    ] {
        let mut command = vec!["--output", "json"];
        command.extend(arguments);
        let output = atla(&config, &command);
        assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
        let deleted: serde_json::Value =
            serde_json::from_str(&stdout(&output)).expect("structured deletion result");
        assert_eq!(deleted["schemaVersion"], 1);
        assert_eq!(deleted["deleted"], true);
    }

    let requests = server
        .received_requests()
        .await
        .expect("request recording should be enabled");
    assert_eq!(requests.len(), 6);
    for pair in requests.chunks_exact(2) {
        let first_query = pair[0]
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();
        let second_query = pair[1]
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();
        assert!(
            first_query.is_empty(),
            "unexpected false options: {first_query:?}"
        );
        assert_eq!(second_query, vec![("purge".to_owned(), "true".to_owned())]);
    }
    server.verify().await;
}
