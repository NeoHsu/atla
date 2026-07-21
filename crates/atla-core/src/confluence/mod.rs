use atla_confluence_api::Client as GeneratedClient;
use atla_confluence_v1_api::Client as GeneratedV1Client;

use crate::client::AtlassianClient;

mod attachments;
mod blog;
mod comments;
mod labels;
pub mod models;
mod pages;
mod search;
mod spaces;
pub mod util;

pub use models::*;

#[derive(Debug, Clone)]
pub struct ConfluenceClient {
    raw_client: AtlassianClient,
    generated: GeneratedClient,
    generated_v1: GeneratedV1Client,
}

impl ConfluenceClient {
    pub fn new(client: AtlassianClient) -> Self {
        let http_client = client.authed_http_client();
        let base_url_v2 = format!("{}/wiki/api/v2", client.instance().base_url);
        let base_url_v1 = client.instance().base_url.clone();

        let generated = GeneratedClient::new_with_client(&base_url_v2, http_client.clone());
        let generated_v1 = GeneratedV1Client::new_with_client(&base_url_v1, http_client);

        Self {
            raw_client: client,
            generated,
            generated_v1,
        }
    }

    pub fn instance_url(&self) -> &str {
        &self.raw_client.instance().base_url
    }
}

#[cfg(test)]
mod tests {
    use super::util::{limit_i32, parse_i64_id};
    use super::*;
    use atla_confluence_api::types as generated_models;
    use atla_confluence_v1_api::types as generated_v1_models;
    use serde_json::json;

    #[test]
    fn parses_space_page() {
        let page: ConfluenceSpacePage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "12345",
                        "key": "DEV",
                        "name": "Development",
                        "type": "global",
                        "status": "current",
                        "homepageId": "67890",
                        "currentActiveAlias": "DEV"
                    }
                ]
            }"#,
        )
        .expect("parse space page");

        assert_eq!(page.results[0].key.as_deref(), Some("DEV"));
        assert_eq!(page.results[0].space_type.as_deref(), Some("global"));
        assert_eq!(page.results[0].homepage_id.as_deref(), Some("67890"));
    }

    #[test]
    fn builds_generated_create_space_request() {
        let request = ConfluenceSpaceCreate {
            key: Some("DEV".to_owned()),
            alias: None,
            name: "Development".to_owned(),
            description: Some("Team docs".to_owned()),
            private: true,
        }
        .to_generated();

        assert_eq!(request.name, "Development");
        assert_eq!(request.key.as_deref(), Some("DEV"));
        assert_eq!(request.create_private_space, Some(true));
        let description = request.description.expect("description");
        assert_eq!(description.value.as_deref(), Some("Team docs"));
        assert_eq!(description.representation.as_deref(), Some("plain"));
    }

    #[test]
    fn builds_v1_update_space_request() {
        let request = ConfluenceSpaceUpdate {
            key: "DEV".to_owned(),
            name: Some("Development".to_owned()),
            description: Some("Team docs".to_owned()),
        }
        .to_v1_update_request();

        assert_eq!(
            request.name.as_deref().map(|name| name.as_str()),
            Some("Development")
        );
        let description = request.description.expect("description");
        let description = description.0.expect("description payload");
        assert_eq!(description.plain.value.as_deref(), Some("Team docs"));
        assert_eq!(description.plain.representation.as_deref(), Some("plain"));
    }

    #[test]
    fn converts_created_space_response() {
        let generated: generated_models::CreateSpaceResponse = serde_json::from_value(json!({
            "id": "12345",
            "key": "DEV",
            "name": "Development",
            "type": "global",
            "status": "current",
            "homepageId": "67890",
            "currentActiveAlias": "DEV"
        }))
        .expect("parse generated space");
        let space = ConfluenceSpace::from(generated);

        assert_eq!(space.key.as_deref(), Some("DEV"));
        assert_eq!(space.space_type.as_deref(), Some("global"));
        assert_eq!(space.status.as_deref(), Some("current"));
        assert_eq!(space.homepage_id.as_deref(), Some("67890"));
    }

    #[test]
    fn parses_page_page() {
        let page: ConfluencePagePage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "111",
                        "status": "current",
                        "title": "Runbook",
                        "spaceId": "12345",
                        "parentId": "100",
                        "authorId": "abc",
                        "createdAt": "2026-05-17T00:00:00.000Z",
                        "version": {
                            "number": 3,
                            "message": "Update",
                            "createdAt": "2026-05-17T01:00:00.000Z"
                        }
                    }
                ]
            }"#,
        )
        .expect("parse page page");

        let page = &page.results[0];
        assert_eq!(page.id.as_deref(), Some("111"));
        assert_eq!(page.title.as_deref(), Some("Runbook"));
        assert_eq!(page.space_id.as_deref(), Some("12345"));
        assert_eq!(
            page.version.as_ref().and_then(|version| version.number),
            Some(3)
        );
    }

    #[test]
    fn converts_direct_child_content_node() {
        let generated: generated_models::ChildrenResponse = serde_json::from_value(json!({
            "id": "333",
            "title": "Folder",
            "type": "folder",
            "spaceId": "12345",
            "childPosition": 4
        }))
        .expect("parse generated child");
        let child = ConfluenceContentNode::from(generated);

        assert_eq!(child.id.as_deref(), Some("333"));
        assert_eq!(child.content_type.as_deref(), Some("folder"));
        assert_eq!(child.space_id.as_deref(), Some("12345"));
        assert_eq!(child.child_position, Some(4));
        assert_eq!(child.depth, None);
    }

    #[test]
    fn converts_descendant_content_node() {
        let generated: generated_models::DescendantsResponse = serde_json::from_value(json!({
            "id": "444",
            "title": "Whiteboard",
            "type": "whiteboard",
            "parentId": "333",
            "depth": 2,
            "childPosition": 1
        }))
        .expect("parse generated descendant");
        let descendant = ConfluenceContentNode::from(generated);

        assert_eq!(descendant.id.as_deref(), Some("444"));
        assert_eq!(descendant.content_type.as_deref(), Some("whiteboard"));
        assert_eq!(descendant.parent_id.as_deref(), Some("333"));
        assert_eq!(descendant.depth, Some(2));
        assert_eq!(descendant.child_position, Some(1));
    }

    #[test]
    fn parses_blog_post_page() {
        let page: ConfluenceBlogPostPage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "222",
                        "status": "current",
                        "title": "Release Notes",
                        "spaceId": "12345",
                        "authorId": "abc",
                        "createdAt": "2026-05-17T00:00:00.000Z",
                        "version": {
                            "number": 2,
                            "message": "Publish",
                            "createdAt": "2026-05-17T01:00:00.000Z"
                        }
                    }
                ]
            }"#,
        )
        .expect("parse blog post page");

        let post = &page.results[0];
        assert_eq!(post.id.as_deref(), Some("222"));
        assert_eq!(post.title.as_deref(), Some("Release Notes"));
        assert_eq!(post.space_id.as_deref(), Some("12345"));
        assert_eq!(
            post.version.as_ref().and_then(|version| version.number),
            Some(2)
        );
    }

    #[test]
    fn parses_label_page() {
        let page: ConfluenceLabelPage = serde_json::from_str(
            r#"{
                "results": [
                    { "id": "1", "name": "runbook", "prefix": "global" }
                ]
            }"#,
        )
        .expect("parse labels");

        assert_eq!(page.results[0].name.as_deref(), Some("runbook"));
        assert_eq!(page.results[0].prefix.as_deref(), Some("global"));
    }

    #[test]
    fn builds_generated_footer_comment_request() {
        let reply = ConfluenceCommentCreate {
            content_id: "111".to_owned(),
            parent_comment_id: Some("222".to_owned()),
            body: "<p>Looks good</p>".to_owned(),
            representation: ConfluenceBodyRepresentation::Storage,
        };

        let generated_page_reply = reply.to_generated_page_footer();
        assert_eq!(generated_page_reply.page_id, None);
        assert_eq!(
            generated_page_reply.parent_comment_id.as_deref(),
            Some("222")
        );
        let Some(body) = generated_page_reply.body else {
            panic!("expected body");
        };
        let generated_models::CreateFooterCommentModelBody::BodyWrite(body) = body else {
            panic!("expected comment body write");
        };
        assert_eq!(body.value.as_deref(), Some("<p>Looks good</p>"));

        let generated_blog_reply = reply.to_generated_blog_footer();
        assert_eq!(generated_blog_reply.blog_post_id, None);
        assert_eq!(
            generated_blog_reply.parent_comment_id.as_deref(),
            Some("222")
        );

        let top_level = ConfluenceCommentCreate {
            parent_comment_id: None,
            ..reply
        };
        assert_eq!(
            top_level.to_generated_page_footer().page_id.as_deref(),
            Some("111")
        );
        assert_eq!(
            top_level.to_generated_blog_footer().blog_post_id.as_deref(),
            Some("111")
        );
    }

    #[test]
    fn builds_generated_blog_post_update_request() {
        let request = ConfluenceBlogPostUpdate {
            id: "222".to_owned(),
            status: ConfluenceContentStatus::Current,
            title: "Release Notes".to_owned(),
            space_id: Some("12345".to_owned()),
            body: "<p>Updated</p>".to_owned(),
            representation: ConfluenceBodyRepresentation::Storage,
            version: 3,
            message: Some("Update".to_owned()),
        }
        .to_generated();

        assert_eq!(request.id, "222");
        assert_eq!(request.title, "Release Notes");
        assert_eq!(request.space_id.as_deref(), Some("12345"));
        assert_eq!(request.version.number, Some(3));
        assert_eq!(request.version.message.as_deref(), Some("Update"));
    }

    #[test]
    fn builds_generated_create_page_request() {
        let request = ConfluencePageCreate {
            space_id: "12345".to_owned(),
            title: "Meeting Notes".to_owned(),
            parent_id: Some("100".to_owned()),
            body: Some("<p>Hello</p>".to_owned()),
            representation: ConfluenceBodyRepresentation::Storage,
            status: ConfluenceContentStatus::Current,
            private: None,
            root_level: None,
        }
        .to_generated();

        assert_eq!(request.space_id, "12345");
        assert_eq!(request.title.as_deref(), Some("Meeting Notes"));
        assert_eq!(request.parent_id.as_deref(), Some("100"));

        let Some(body) = request.body else {
            panic!("expected generated body");
        };
        let generated_models::CreatePageBodyBody::BodyWrite(body) = body else {
            panic!("expected page body write");
        };
        assert_eq!(body.value.as_deref(), Some("<p>Hello</p>"));
        assert_eq!(
            body.representation,
            Some(generated_models::PageBodyWriteRepresentation::Storage)
        );
    }

    #[test]
    fn parses_search_page() {
        let page: ConfluenceSearchPage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "title": "Runbook",
                        "url": "/wiki/spaces/DEV/pages/111/Runbook",
                        "content": {
                            "id": "111",
                            "type": "page",
                            "status": "current",
                            "title": "Runbook"
                        }
                    }
                ],
                "size": 1,
                "totalSize": 1
            }"#,
        )
        .expect("parse search page");

        let result = &page.results[0];
        assert_eq!(result.title.as_deref(), Some("Runbook"));
        assert_eq!(
            result
                .content
                .as_ref()
                .and_then(|content| content.content_type.as_deref()),
            Some("page")
        );
    }

    #[test]
    fn parses_attachment_page() {
        let page: ConfluenceAttachmentPage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "att123",
                        "status": "current",
                        "title": "diagram.png",
                        "pageId": "111",
                        "mediaType": "image/png",
                        "fileSize": 2048,
                        "downloadLink": "/download/attachments/111/diagram.png",
                        "version": {
                            "number": 1
                        }
                    }
                ]
            }"#,
        )
        .expect("parse attachment page");

        let attachment = &page.results[0];
        assert_eq!(attachment.id.as_deref(), Some("att123"));
        assert_eq!(attachment.media_type.as_deref(), Some("image/png"));
        assert_eq!(attachment.file_size, Some(2048));
    }

    #[test]
    fn converts_v1_search_page() {
        let generated: generated_v1_models::SearchPageResponseSearchResult =
            serde_json::from_value(json!({
                "results": [{
                    "title": "Runbook",
                    "excerpt": "Useful page",
                    "url": "/wiki/spaces/DEV/pages/111/Runbook",
                    "content": {
                        "id": "111",
                        "type": "page",
                        "status": "current",
                        "title": "Runbook"
                    }
                }],
                "size": 1,
                "totalSize": 1
            }))
            .expect("parse generated search page");
        let page = generated;

        let page = ConfluenceSearchPage::from(page);
        let result = &page.results[0];
        assert_eq!(page.total_size, Some(1));
        assert_eq!(result.title.as_deref(), Some("Runbook"));
        assert_eq!(
            result
                .content
                .as_ref()
                .and_then(|content| content.content_type.as_deref()),
            Some("page")
        );
    }

    #[test]
    fn converts_v1_attachment_upload_result() {
        let generated: generated_v1_models::Content = serde_json::from_value(json!({
            "id": "att123",
            "type": "attachment",
            "status": "current",
            "title": "diagram.png",
            "version": { "number": 2 },
            "_links": {
                "download": "/download/attachments/111/diagram.png"
            }
        }))
        .expect("parse generated v1 attachment");
        let attachment = ConfluenceAttachment::from(generated);

        assert_eq!(attachment.id.as_deref(), Some("att123"));
        assert_eq!(attachment.title.as_deref(), Some("diagram.png"));
        assert_eq!(
            attachment.download_link.as_deref(),
            Some("/download/attachments/111/diagram.png")
        );
        assert_eq!(
            attachment
                .version
                .as_ref()
                .and_then(|version| version.number),
            Some(2)
        );
    }

    #[test]
    fn parses_space_with_description() {
        let space: ConfluenceSpace = serde_json::from_str(
            r#"{
                "id": "12345",
                "key": "DEV",
                "name": "Development",
                "type": "global",
                "status": "current",
                "homepageId": "67890",
                "currentActiveAlias": "DEV",
                "spaceOwnerId": "owner-123",
                "description": {
                    "plain": {
                        "value": "Team docs"
                    }
                }
            }"#,
        )
        .expect("parse space");

        assert_eq!(space.id.as_deref(), Some("12345"));
        assert_eq!(space.key.as_deref(), Some("DEV"));
        assert_eq!(space.name.as_deref(), Some("Development"));
        assert_eq!(space.space_type.as_deref(), Some("global"));
        assert_eq!(space.status.as_deref(), Some("current"));
        assert_eq!(space.homepage_id.as_deref(), Some("67890"));
        assert_eq!(space.current_active_alias.as_deref(), Some("DEV"));
        assert_eq!(space.space_owner_id.as_deref(), Some("owner-123"));
    }

    #[test]
    fn converts_space_from_generated_space() {
        let generated: generated_models::SpaceBulk = serde_json::from_value(json!({
            "id": "12345",
            "key": "DEV",
            "name": "Development",
            "type": "global",
            "status": "current",
            "homepageId": "67890",
            "currentActiveAlias": "dev",
            "spaceOwnerId": "owner-456"
        }))
        .expect("parse generated bulk space");
        let space = ConfluenceSpace::from(generated);

        assert_eq!(space.id.as_deref(), Some("12345"));
        assert_eq!(space.key.as_deref(), Some("DEV"));
        assert_eq!(space.name.as_deref(), Some("Development"));
        assert_eq!(space.space_type.as_deref(), Some("global"));
        assert_eq!(space.status.as_deref(), Some("current"));
        assert_eq!(space.homepage_id.as_deref(), Some("67890"));
        assert_eq!(space.current_active_alias.as_deref(), Some("dev"));
        assert_eq!(space.space_owner_id.as_deref(), Some("owner-456"));
    }

    #[test]
    fn parses_single_page_with_body() {
        let page: ConfluencePage = serde_json::from_str(
            r#"{
                "id": "111",
                "status": "current",
                "title": "Runbook",
                "spaceId": "12345",
                "parentId": "100",
                "authorId": "abc",
                "createdAt": "2026-05-17T00:00:00.000Z",
                "body": "<p>Hello</p>",
                "version": {
                    "number": 3,
                    "message": "Update"
                }
            }"#,
        )
        .expect("parse page");

        assert_eq!(page.id.as_deref(), Some("111"));
        assert_eq!(page.status.as_deref(), Some("current"));
        assert_eq!(page.title.as_deref(), Some("Runbook"));
        assert_eq!(page.space_id.as_deref(), Some("12345"));
        assert_eq!(page.parent_id.as_deref(), Some("100"));
        assert_eq!(page.author_id.as_deref(), Some("abc"));
        assert_eq!(page.created_at.as_deref(), Some("2026-05-17T00:00:00.000Z"));
        assert_eq!(page.body.as_deref(), Some("<p>Hello</p>"));
        assert_eq!(
            page.version.as_ref().and_then(|version| version.number),
            Some(3)
        );
    }

    #[test]
    fn parses_single_blog_post() {
        let post: ConfluenceBlogPost = serde_json::from_str(
            r#"{
                "id": "222",
                "status": "current",
                "title": "Release Notes",
                "spaceId": "12345",
                "authorId": "abc",
                "createdAt": "2026-05-17T00:00:00.000Z",
                "body": "<p>Shipped</p>"
            }"#,
        )
        .expect("parse blog post");

        assert_eq!(post.id.as_deref(), Some("222"));
        assert_eq!(post.title.as_deref(), Some("Release Notes"));
        assert_eq!(post.space_id.as_deref(), Some("12345"));
        assert_eq!(post.author_id.as_deref(), Some("abc"));
        assert_eq!(post.body.as_deref(), Some("<p>Shipped</p>"));
    }

    #[test]
    fn parses_comment_page() {
        let page: ConfluenceCommentPage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "c1",
                        "status": "current",
                        "title": "First comment",
                        "pageId": "111",
                        "body": "<p>Looks good</p>",
                        "version": {
                            "number": 1,
                            "createdAt": "2026-05-17T02:00:00.000Z"
                        }
                    },
                    {
                        "id": "c2",
                        "status": "current",
                        "title": "Reply",
                        "blogPostId": "222",
                        "parentCommentId": "c1",
                        "body": "<p>Approved</p>"
                    }
                ]
            }"#,
        )
        .expect("parse comment page");

        assert_eq!(page.results.len(), 2);
        assert_eq!(page.results[0].id.as_deref(), Some("c1"));
        assert_eq!(page.results[0].body.as_deref(), Some("<p>Looks good</p>"));
        assert_eq!(page.results[1].blog_post_id.as_deref(), Some("222"));
        assert_eq!(page.results[1].parent_comment_id.as_deref(), Some("c1"));
    }

    #[test]
    fn parses_single_comment() {
        let comment: ConfluenceComment = serde_json::from_str(
            r#"{
                "id": "c1",
                "status": "current",
                "title": "Looks good",
                "pageId": "111",
                "parentCommentId": "c0",
                "body": "<p>Ship it</p>",
                "version": {
                    "number": 4,
                    "message": "Edited"
                }
            }"#,
        )
        .expect("parse comment");

        assert_eq!(comment.id.as_deref(), Some("c1"));
        assert_eq!(comment.page_id.as_deref(), Some("111"));
        assert_eq!(comment.parent_comment_id.as_deref(), Some("c0"));
        assert_eq!(comment.body.as_deref(), Some("<p>Ship it</p>"));
        assert_eq!(
            comment.version.as_ref().and_then(|version| version.number),
            Some(4)
        );
        assert_eq!(
            comment
                .version
                .as_ref()
                .and_then(|version| version.message.as_deref()),
            Some("Edited")
        );
    }

    #[test]
    fn parses_attachment_with_download_link() {
        let attachment: ConfluenceAttachment = serde_json::from_str(
            r#"{
                "id": "att123",
                "status": "current",
                "title": "diagram.png",
                "pageId": "111",
                "mediaType": "image/png",
                "downloadLink": "/download/attachments/111/diagram.png"
            }"#,
        )
        .expect("parse attachment");

        assert_eq!(attachment.id.as_deref(), Some("att123"));
        assert_eq!(attachment.page_id.as_deref(), Some("111"));
        assert_eq!(attachment.media_type.as_deref(), Some("image/png"));
        assert_eq!(
            attachment.download_link.as_deref(),
            Some("/download/attachments/111/diagram.png")
        );
    }

    #[test]
    fn parses_attachment_download_link_from_links() {
        let generated: generated_models::GetAttachmentByIdResponse = serde_json::from_str(
            r#"{
                "id": "att123",
                "status": "current",
                "title": "diagram.png",
                "_links": {
                    "download": "/download/attachments/111/diagram.png"
                }
            }"#,
        )
        .expect("parse attachment");
        let attachment = ConfluenceAttachment::from(generated);

        assert_eq!(
            attachment.download_link.as_deref(),
            Some("/download/attachments/111/diagram.png")
        );
    }

    #[test]
    fn parses_att_prefixed_numeric_ids() {
        assert_eq!(
            parse_i64_id("att2513502311").expect("attachment id"),
            2513502311
        );
        assert_eq!(parse_i64_id("2513502311").expect("plain id"), 2513502311);
    }

    #[test]
    fn parses_search_page_empty() {
        let page: ConfluenceSearchPage = serde_json::from_str(
            r#"{
                "results": [],
                "size": 0,
                "totalSize": 0
            }"#,
        )
        .expect("parse empty search page");

        assert!(page.results.is_empty());
        assert_eq!(page.size, Some(0));
        assert_eq!(page.total_size, Some(0));
    }

    #[test]
    fn parses_search_page_with_excerpt() {
        let page: ConfluenceSearchPage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "title": "Runbook",
                        "url": "/wiki/spaces/DEV/pages/111/Runbook",
                        "excerpt": "Useful page",
                        "content": {
                            "id": "111",
                            "type": "page",
                            "status": "current",
                            "title": "Runbook"
                        }
                    }
                ],
                "size": 1,
                "totalSize": 1
            }"#,
        )
        .expect("parse search page with excerpt");

        assert_eq!(page.results.len(), 1);
        assert_eq!(page.results[0].excerpt.as_deref(), Some("Useful page"));
        assert_eq!(
            page.results[0]
                .content
                .as_ref()
                .and_then(|content| content.id.as_deref()),
            Some("111")
        );
    }

    #[test]
    fn parses_label_page_multiple() {
        let page: ConfluenceLabelPage = serde_json::from_str(
            r#"{
                "results": [
                    { "id": "1", "name": "runbook", "prefix": "global" },
                    { "id": "2", "name": "docs", "prefix": "my" }
                ]
            }"#,
        )
        .expect("parse labels");

        assert_eq!(page.results.len(), 2);
        assert_eq!(page.results[0].name.as_deref(), Some("runbook"));
        assert_eq!(page.results[0].prefix.as_deref(), Some("global"));
        assert_eq!(page.results[1].name.as_deref(), Some("docs"));
        assert_eq!(page.results[1].prefix.as_deref(), Some("my"));
    }

    #[test]
    fn parses_attachment_page_empty() {
        let page: ConfluenceAttachmentPage = serde_json::from_str(
            r#"{
                "results": []
            }"#,
        )
        .expect("parse empty attachment page");

        assert!(page.results.is_empty());
    }

    #[test]
    fn parses_content_tree_page() {
        let page: ConfluenceContentTreePage = serde_json::from_str(
            r#"{
                "results": [
                    {
                        "id": "111",
                        "status": "current",
                        "title": "Runbook",
                        "type": "page",
                        "spaceId": "12345",
                        "childPosition": 1
                    },
                    {
                        "id": "112",
                        "status": "current",
                        "title": "Team Folder",
                        "type": "folder",
                        "spaceId": "12345",
                        "childPosition": 2
                    }
                ]
            }"#,
        )
        .expect("parse content tree page");

        assert_eq!(page.results.len(), 2);
        assert_eq!(page.results[0].content_type.as_deref(), Some("page"));
        assert_eq!(page.results[1].content_type.as_deref(), Some("folder"));
        assert_eq!(page.results[1].child_position, Some(2));
    }

    #[test]
    fn parses_page_version() {
        let page: ConfluencePage = serde_json::from_str(
            r#"{
                "id": "111",
                "version": {
                    "number": 7,
                    "message": "Promoted",
                    "createdAt": "2026-05-17T01:00:00.000Z"
                }
            }"#,
        )
        .expect("parse page version");

        let version = page.version.expect("page version");
        assert_eq!(version.number, Some(7));
        assert_eq!(version.message.as_deref(), Some("Promoted"));
        assert_eq!(
            version.created_at.as_deref(),
            Some("2026-05-17T01:00:00.000Z")
        );
    }

    #[test]
    fn parses_blog_post_with_version() {
        let post: ConfluenceBlogPost = serde_json::from_str(
            r#"{
                "id": "222",
                "version": {
                    "number": 2,
                    "message": "Published",
                    "createdAt": "2026-05-17T03:00:00.000Z"
                }
            }"#,
        )
        .expect("parse blog post version");

        let version = post.version.expect("blog post version");
        assert_eq!(version.number, Some(2));
        assert_eq!(version.message.as_deref(), Some("Published"));
        assert_eq!(
            version.created_at.as_deref(),
            Some("2026-05-17T03:00:00.000Z")
        );
    }

    #[test]
    fn space_search_defaults() {
        let search = ConfluenceSpaceSearch::default();

        assert_eq!(search.key, None);
        assert_eq!(search.limit, 25);
    }

    #[test]
    fn confluence_search_clamps_limit() {
        let search = ConfluenceSearch {
            cql: "type = page".to_owned(),
            limit: u32::MAX,
            start: 0,
        };

        assert_eq!(search.limit, u32::MAX);
        assert_eq!(limit_i32(search.limit), i32::MAX);
    }

    #[test]
    fn converts_page_from_generated() {
        let generated: generated_models::CreatePageResponse = serde_json::from_value(json!({
            "id": "111",
            "status": "current",
            "title": "My Page",
            "spaceId": "12345",
            "parentId": "100",
            "authorId": "abc",
            "createdAt": "2026-05-17T00:00:00Z",
            "version": {
                "number": 3,
                "message": "Updated",
                "createdAt": "2026-05-17T01:00:00Z"
            },
            "body": {
                "storage": {
                    "value": "<p>Hello</p>"
                }
            }
        }))
        .expect("parse generated page");
        let page = ConfluencePage::from(generated);

        assert_eq!(page.id.as_deref(), Some("111"));
        assert_eq!(page.status.as_deref(), Some("current"));
        assert_eq!(page.title.as_deref(), Some("My Page"));
        assert_eq!(page.space_id.as_deref(), Some("12345"));
        assert_eq!(page.parent_id.as_deref(), Some("100"));
        assert_eq!(page.author_id.as_deref(), Some("abc"));
        assert_eq!(page.body.as_deref(), Some("<p>Hello</p>"));
        assert_eq!(
            page.version.as_ref().and_then(|version| version.number),
            Some(3)
        );
        assert_eq!(
            page.version
                .as_ref()
                .and_then(|version| version.message.as_deref()),
            Some("Updated")
        );
    }

    #[test]
    fn converts_blog_post_from_generated() {
        let generated: generated_models::CreateBlogPostResponse = serde_json::from_value(json!({
            "id": "222",
            "status": "current",
            "title": "Release Notes",
            "spaceId": "12345",
            "authorId": "abc",
            "createdAt": "2026-05-17T00:00:00Z",
            "version": {
                "number": 2,
                "message": "Published"
            },
            "body": {
                "storage": {
                    "value": "<p>Shipped</p>"
                }
            }
        }))
        .expect("parse generated blog post");
        let post = ConfluenceBlogPost::from(generated);

        assert_eq!(post.id.as_deref(), Some("222"));
        assert_eq!(post.status.as_deref(), Some("current"));
        assert_eq!(post.title.as_deref(), Some("Release Notes"));
        assert_eq!(post.space_id.as_deref(), Some("12345"));
        assert_eq!(post.author_id.as_deref(), Some("abc"));
        assert_eq!(post.body.as_deref(), Some("<p>Shipped</p>"));
        assert_eq!(
            post.version.as_ref().and_then(|version| version.number),
            Some(2)
        );
        assert_eq!(
            post.version
                .as_ref()
                .and_then(|version| version.message.as_deref()),
            Some("Published")
        );
    }

    #[test]
    fn converts_space_comment_page() {
        let generated: generated_models::MultiEntityResultPageCommentModel =
            serde_json::from_value(json!({
                "results": [{
                    "id": "c1",
                    "status": "current",
                    "title": "First comment",
                    "pageId": "111",
                    "version": { "number": 1 },
                    "body": {
                        "storage": {
                            "value": "<p>Looks good</p>"
                        }
                    }
                }]
            }))
            .expect("parse generated comments page");
        let page = ConfluenceCommentPage::from(generated);

        assert_eq!(page.results.len(), 1);
        let comment = &page.results[0];
        assert_eq!(comment.id.as_deref(), Some("c1"));
        assert_eq!(comment.page_id.as_deref(), Some("111"));
        assert_eq!(comment.body.as_deref(), Some("<p>Looks good</p>"));
        assert_eq!(
            comment.version.as_ref().and_then(|version| version.number),
            Some(1)
        );
    }
}
