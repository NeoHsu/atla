use atla_confluence_api::apis as generated_apis;
use atla_confluence_v1_api::apis as generated_v1_apis;

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
    generated: generated_apis::configuration::Configuration,
    generated_v1: generated_v1_apis::configuration::Configuration,
}

impl ConfluenceClient {
    pub fn new(client: AtlassianClient) -> Self {
        let generated = generated_apis::configuration::Configuration {
            base_path: format!("{}/wiki/api/v2", client.instance().base_url),
            user_agent: Some("atla".to_owned()),
            basic_auth: Some((client.email().to_owned(), Some(client.token().to_owned()))),
            ..Default::default()
        };
        let generated_v1 = generated_v1_apis::configuration::Configuration {
            base_path: client.instance().base_url.clone(),
            user_agent: Some("atla".to_owned()),
            basic_auth: Some((client.email().to_owned(), Some(client.token().to_owned()))),
            ..Default::default()
        };

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
    use super::util::limit_i32;
    use super::*;
    use atla_confluence_api::models as generated_models;
    use atla_confluence_v1_api::models as generated_v1_models;

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

        assert_eq!(request.name, Some(Some("Development".to_owned())));
        let description = request
            .description
            .expect("description")
            .expect("description payload");
        assert_eq!(description.plain.value.as_deref(), Some("Team docs"));
        assert_eq!(description.plain.representation.as_deref(), Some("plain"));
    }

    #[test]
    fn converts_created_space_response() {
        let space = ConfluenceSpace::from(generated_models::CreateSpace201Response {
            id: Some("12345".to_owned()),
            key: Some("DEV".to_owned()),
            name: Some("Development".to_owned()),
            r#type: Some(generated_models::SpaceType::Global),
            status: Some(generated_models::SpaceStatus::Current),
            homepage_id: Some("67890".to_owned()),
            current_active_alias: Some("DEV".to_owned()),
            ..generated_models::CreateSpace201Response::new()
        });

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
        let child = ConfluenceContentNode::from(generated_models::ChildrenResponse {
            id: Some("333".to_owned()),
            status: None,
            title: Some("Folder".to_owned()),
            r#type: Some("folder".to_owned()),
            space_id: Some("12345".to_owned()),
            child_position: Some(Some(4)),
        });

        assert_eq!(child.id.as_deref(), Some("333"));
        assert_eq!(child.content_type.as_deref(), Some("folder"));
        assert_eq!(child.space_id.as_deref(), Some("12345"));
        assert_eq!(child.child_position, Some(4));
        assert_eq!(child.depth, None);
    }

    #[test]
    fn converts_descendant_content_node() {
        let descendant = ConfluenceContentNode::from(generated_models::DescendantsResponse {
            id: Some("444".to_owned()),
            status: None,
            title: Some("Whiteboard".to_owned()),
            r#type: Some("whiteboard".to_owned()),
            parent_id: Some("333".to_owned()),
            depth: Some(2),
            child_position: Some(Some(1)),
        });

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
        let comment = ConfluenceCommentCreate {
            content_id: "111".to_owned(),
            parent_comment_id: Some("222".to_owned()),
            body: "<p>Looks good</p>".to_owned(),
            representation: ConfluenceBodyRepresentation::Storage,
        };

        let generated = comment.to_generated_page_footer();

        assert_eq!(generated.page_id.as_deref(), Some("111"));
        assert_eq!(generated.parent_comment_id.as_deref(), Some("222"));
        let Some(body) = generated.body else {
            panic!("expected body");
        };
        let generated_models::CreateFooterCommentModelBody::CommentBodyWrite(body) = *body else {
            panic!("expected comment body write");
        };
        assert_eq!(body.value.as_deref(), Some("<p>Looks good</p>"));
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
        let generated_models::CreatePageRequestBody::PageBodyWrite(body) = *body else {
            panic!("expected page body write");
        };
        assert_eq!(body.value.as_deref(), Some("<p>Hello</p>"));
        assert_eq!(
            body.representation,
            Some(generated_models::page_body_write::Representation::Storage)
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
        let page = generated_v1_models::SearchPageResponseSearchResult::new(
            vec![generated_v1_models::SearchResult {
                title: "Runbook".to_owned(),
                excerpt: "Useful page".to_owned(),
                url: "/wiki/spaces/DEV/pages/111/Runbook".to_owned(),
                content: Some(Box::new(generated_v1_models::Content {
                    id: Some("111".to_owned()),
                    r#type: "page".to_owned(),
                    status: "current".to_owned(),
                    title: Some("Runbook".to_owned()),
                    version: None,
                    _links: None,
                })),
            }],
            1,
            1,
        );

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
        let mut links = std::collections::HashMap::new();
        links.insert(
            "download".to_owned(),
            "/download/attachments/111/diagram.png".to_owned(),
        );

        let attachment = ConfluenceAttachment::from(generated_v1_models::Content {
            id: Some("att123".to_owned()),
            r#type: "attachment".to_owned(),
            status: "current".to_owned(),
            title: Some("diagram.png".to_owned()),
            version: Some(Box::new(generated_v1_models::Version::new(2))),
            _links: Some(links),
        });

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
    }

    #[test]
    fn converts_space_from_generated_space() {
        let space = ConfluenceSpace::from(generated_models::SpaceBulk {
            id: Some("12345".to_owned()),
            key: Some("DEV".to_owned()),
            name: Some("Development".to_owned()),
            r#type: Some(generated_models::SpaceType::Global),
            status: Some(generated_models::SpaceStatus::Current),
            homepage_id: Some("67890".to_owned()),
            current_active_alias: Some("dev".to_owned()),
            ..generated_models::SpaceBulk::new()
        });

        assert_eq!(space.id.as_deref(), Some("12345"));
        assert_eq!(space.key.as_deref(), Some("DEV"));
        assert_eq!(space.name.as_deref(), Some("Development"));
        assert_eq!(space.space_type.as_deref(), Some("global"));
        assert_eq!(space.status.as_deref(), Some("current"));
        assert_eq!(space.homepage_id.as_deref(), Some("67890"));
        assert_eq!(space.current_active_alias.as_deref(), Some("dev"));
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
        };

        assert_eq!(search.limit, u32::MAX);
        assert_eq!(limit_i32(search.limit), i32::MAX);
    }

    #[test]
    fn converts_page_from_generated() {
        let mut version = generated_models::Version::new();
        version.number = Some(3);
        version.message = Some("Updated".to_owned());
        version.created_at = Some("2026-05-17T01:00:00Z".parse().expect("parse created_at"));

        let mut storage = generated_models::BodyType::new();
        storage.value = Some("<p>Hello</p>".to_owned());

        let mut body = generated_models::BodySingle::new();
        body.storage = Some(Box::new(storage));

        let page = ConfluencePage::from(generated_models::CreatePage200Response {
            id: Some("111".to_owned()),
            status: Some(generated_models::ContentStatus::Current),
            title: Some("My Page".to_owned()),
            space_id: Some("12345".to_owned()),
            parent_id: Some("100".to_owned()),
            author_id: Some("abc".to_owned()),
            created_at: Some("2026-05-17T00:00:00Z".parse().expect("parse created_at")),
            version: Some(Box::new(version)),
            body: Some(Box::new(body)),
            ..generated_models::CreatePage200Response::new()
        });

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
        let mut version = generated_models::Version::new();
        version.number = Some(2);
        version.message = Some("Published".to_owned());

        let mut storage = generated_models::BodyType::new();
        storage.value = Some("<p>Shipped</p>".to_owned());

        let mut body = generated_models::BodySingle::new();
        body.storage = Some(Box::new(storage));

        let post = ConfluenceBlogPost::from(generated_models::CreateBlogPost200Response {
            id: Some("222".to_owned()),
            status: Some(generated_models::BlogPostContentStatus::Current),
            title: Some("Release Notes".to_owned()),
            space_id: Some("12345".to_owned()),
            author_id: Some("abc".to_owned()),
            created_at: Some("2026-05-17T00:00:00Z".parse().expect("parse created_at")),
            version: Some(Box::new(version)),
            body: Some(Box::new(body)),
            ..generated_models::CreateBlogPost200Response::new()
        });

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
        let mut version = generated_models::Version::new();
        version.number = Some(1);

        let mut storage = generated_models::BodyType::new();
        storage.value = Some("<p>Looks good</p>".to_owned());

        let mut body = generated_models::BodyBulk::new();
        body.storage = Some(Box::new(storage));

        let page =
            ConfluenceCommentPage::from(generated_models::MultiEntityResultPageCommentModel {
                results: Some(vec![generated_models::PageCommentModel {
                    id: Some("c1".to_owned()),
                    status: Some(generated_models::ContentStatus::Current),
                    title: Some("First comment".to_owned()),
                    page_id: Some("111".to_owned()),
                    version: Some(Box::new(version)),
                    body: Some(Box::new(body)),
                    ..generated_models::PageCommentModel::new()
                }]),
                ..generated_models::MultiEntityResultPageCommentModel::new()
            });

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
