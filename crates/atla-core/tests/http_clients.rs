use atla_core::{
    AtlassianClient, AtlassianInstance, ConfluenceClient, ConfluenceCommentSearch,
    ConfluenceLabelSearch, ConfluenceSearch, ConfluenceSpaceSearch, JiraClient, JiraProjectSearch,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn raw_client(server: &MockServer) -> AtlassianClient {
    AtlassianClient::new(
        AtlassianInstance::new(server.uri()),
        "agent@example.com",
        "test-token",
    )
}

#[tokio::test]
async fn jira_generated_project_and_comment_reads_are_exercised() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/project/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "isLast": true,
            "startAt": 0,
            "maxResults": 10,
            "total": 1,
            "values": [{"id": "10000", "key": "PROJ", "name": "Project"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/PROJ-1/comment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "startAt": 0,
            "maxResults": 10,
            "total": 1,
            "comments": [{
                "id": "20000",
                "body": {"type": "doc", "version": 1, "content": []}
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = JiraClient::new(raw_client(&server));
    let projects = client
        .search_projects(&JiraProjectSearch {
            start_at: 0,
            max_results: 10,
            query: Some("PROJ".to_owned()),
        })
        .await
        .expect("project search");
    let comments = client
        .list_comments("PROJ-1", 10)
        .await
        .expect("comment list");

    assert_eq!(projects.values[0].key.as_deref(), Some("PROJ"));
    assert_eq!(comments.comments[0].id.as_deref(), Some("20000"));
    server.verify().await;
}

#[tokio::test]
async fn confluence_generated_space_comment_and_label_reads_are_exercised() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
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
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/111/footer-comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "id": "222",
                "status": "current",
                "title": "Re: Runbook",
                "pageId": "111",
                "body": {"storage": {"value": "Looks good"}}
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/111/labels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{"id": "333", "name": "runbook", "prefix": "global"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = ConfluenceClient::new(raw_client(&server));
    let spaces = client
        .list_spaces(&ConfluenceSpaceSearch {
            key: Some("ENG".to_owned()),
            limit: 10,
            cursor: None,
        })
        .await
        .expect("space list");
    let comments = client
        .list_page_comments(&ConfluenceCommentSearch {
            content_id: "111".to_owned(),
            limit: 10,
            cursor: None,
        })
        .await
        .expect("comment list");
    let labels = client
        .list_page_labels(&ConfluenceLabelSearch {
            content_id: "111".to_owned(),
            prefix: None,
            limit: 10,
            cursor: None,
        })
        .await
        .expect("label list");

    assert_eq!(
        spaces.results[0].space_owner_id.as_deref(),
        Some("owner-123")
    );
    assert_eq!(comments.results[0].id.as_deref(), Some("222"));
    assert_eq!(labels.results[0].name.as_deref(), Some("runbook"));
    server.verify().await;
}

#[tokio::test]
async fn confluence_v1_search_read_is_exercised() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wiki/rest/api/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "title": "Runbook",
                "excerpt": "Recovery steps",
                "url": "/wiki/spaces/ENG/pages/111/Runbook",
                "content": {
                    "id": "111",
                    "type": "page",
                    "status": "current",
                    "title": "Runbook"
                }
            }],
            "size": 1,
            "totalSize": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = ConfluenceClient::new(raw_client(&server));
    let results = client
        .search(&ConfluenceSearch {
            cql: "type=page".to_owned(),
            limit: 10,
            start: 0,
        })
        .await
        .expect("CQL search");

    assert_eq!(results.results[0].title.as_deref(), Some("Runbook"));
    assert_eq!(results.total_size, Some(1));
    server.verify().await;
}
