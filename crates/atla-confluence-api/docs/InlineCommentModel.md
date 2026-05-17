# InlineCommentModel

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the comment. | [optional]
**status** | Option<[**models::ContentStatus**](ContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the comment. | [optional]
**blog_post_id** | Option<**String**> | ID of the blog post containing the comment if the comment is on a blog post. | [optional]
**page_id** | Option<**String**> | ID of the page containing the comment if the comment is on a page. | [optional]
**parent_comment_id** | Option<**String**> | ID of the parent comment if the comment is a reply. | [optional]
**version** | Option<[**models::Version**](Version.md)> |  | [optional]
**body** | Option<[**models::BodySingle**](BodySingle.md)> |  | [optional]
**resolution_last_modifier_id** | Option<**String**> | Atlassian Account ID of last person who modified the resolve state of the comment. Null until comment is resolved or reopened. | [optional]
**resolution_last_modified_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Timestamp of the last modification to the comment's resolution status. Null until comment is resolved or reopened. | [optional]
**resolution_status** | Option<[**models::InlineCommentResolutionStatus**](InlineCommentResolutionStatus.md)> |  | [optional]
**properties** | Option<[**models::InlineCommentModelProperties**](InlineCommentModelProperties.md)> |  | [optional]
**operations** | Option<[**models::AttachmentSingleOperations**](AttachmentSingleOperations.md)> |  | [optional]
**likes** | Option<[**models::BlogPostSingleLikes**](BlogPostSingleLikes.md)> |  | [optional]
**versions** | Option<[**models::AttachmentSingleVersions**](AttachmentSingleVersions.md)> |  | [optional]
**_links** | Option<[**models::CommentLinks**](CommentLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
