# CreateFooterCommentModel

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**blog_post_id** | Option<**String**> | ID of the containing blog post, if intending to create a top level footer comment. Do not provide if creating a reply. | [optional]
**page_id** | Option<**String**> | ID of the containing page, if intending to create a top level footer comment. Do not provide if creating a reply. | [optional]
**parent_comment_id** | Option<**String**> | ID of the parent comment, if intending to create a reply. Do not provide if creating a top level comment. | [optional]
**attachment_id** | Option<**String**> | ID of the attachment, if intending to create a comment against an attachment. | [optional]
**custom_content_id** | Option<**String**> | ID of the custom content, if intending to create a comment against a custom content. | [optional]
**body** | Option<[**models::CreateFooterCommentModelBody**](CreateFooterCommentModelBody.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
