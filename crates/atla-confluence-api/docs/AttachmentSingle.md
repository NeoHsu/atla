# AttachmentSingle

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the attachment. | [optional]
**status** | Option<[**models::ContentStatus**](ContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the comment. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the attachment was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**page_id** | Option<**String**> | ID of the containing page.  Note: This is only returned if the attachment has a container that is a page. | [optional]
**blog_post_id** | Option<**String**> | ID of the containing blog post.  Note: This is only returned if the attachment has a container that is a blog post. | [optional]
**custom_content_id** | Option<**String**> | ID of the containing custom content.  Note: This is only returned if the attachment has a container that is custom content. | [optional]
**media_type** | Option<**String**> | Media Type for the attachment. | [optional]
**media_type_description** | Option<**String**> | Media Type description for the attachment. | [optional]
**comment** | Option<**String**> | Comment for the attachment. | [optional]
**file_id** | Option<**String**> | File ID of the attachment. This is the ID referenced in `atlas_doc_format` bodies and is distinct from the attachment ID. | [optional]
**file_size** | Option<**i64**> | File size of the attachment. | [optional]
**webui_link** | Option<**String**> | WebUI link of the attachment. | [optional]
**download_link** | Option<**String**> | Download link of the attachment. | [optional]
**version** | Option<[**models::Version**](Version.md)> |  | [optional]
**labels** | Option<[**models::AttachmentSingleLabels**](AttachmentSingleLabels.md)> |  | [optional]
**properties** | Option<[**models::AttachmentSingleProperties**](AttachmentSingleProperties.md)> |  | [optional]
**operations** | Option<[**models::AttachmentSingleOperations**](AttachmentSingleOperations.md)> |  | [optional]
**versions** | Option<[**models::AttachmentSingleVersions**](AttachmentSingleVersions.md)> |  | [optional]
**_links** | Option<[**models::AttachmentLinks**](AttachmentLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
