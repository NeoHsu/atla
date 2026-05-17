# CreateFooterComment201Response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the comment. | [optional]
**status** | Option<[**models::ContentStatus**](ContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the comment. | [optional]
**blog_post_id** | Option<**String**> | ID of the blog post containing the comment if the comment is on a blog post. | [optional]
**page_id** | Option<**String**> | ID of the page containing the comment if the comment is on a page. | [optional]
**attachment_id** | Option<**String**> | ID of the attachment containing the comment if the comment is on an attachment. | [optional]
**custom_content_id** | Option<**String**> | ID of the custom content containing the comment if the comment is on a custom content. | [optional]
**parent_comment_id** | Option<**String**> | ID of the parent comment if the comment is a reply. | [optional]
**version** | Option<[**models::Version**](Version.md)> |  | [optional]
**properties** | Option<[**models::AttachmentSingleProperties**](AttachmentSingleProperties.md)> |  | [optional]
**operations** | Option<[**models::AttachmentSingleOperations**](AttachmentSingleOperations.md)> |  | [optional]
**likes** | Option<[**models::BlogPostSingleLikes**](BlogPostSingleLikes.md)> |  | [optional]
**versions** | Option<[**models::AttachmentSingleVersions**](AttachmentSingleVersions.md)> |  | [optional]
**body** | Option<[**models::BodySingle**](BodySingle.md)> |  | [optional]
**_links** | Option<[**models::GetAttachmentById200ResponseAllOfLinks**](GetAttachmentById200ResponseAllOfLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
