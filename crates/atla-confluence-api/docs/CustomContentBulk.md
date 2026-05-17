# CustomContentBulk

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the custom content. | [optional]
**r#type** | Option<**String**> | The type of custom content. | [optional]
**status** | Option<[**models::ContentStatus**](ContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the custom content. | [optional]
**space_id** | Option<**String**> | ID of the space the custom content is in.  Note: This is always returned, regardless of if the custom content has a container that is a space. | [optional]
**page_id** | Option<**String**> | ID of the containing page.  Note: This is only returned if the custom content has a container that is a page. | [optional]
**blog_post_id** | Option<**String**> | ID of the containing blog post.  Note: This is only returned if the custom content has a container that is a blog post. | [optional]
**custom_content_id** | Option<**String**> | ID of the containing custom content.  Note: This is only returned if the custom content has a container that is custom content. | [optional]
**author_id** | Option<**String**> | The account ID of the user who created this custom content originally. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the custom content was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**version** | Option<[**models::Version**](Version.md)> |  | [optional]
**body** | Option<[**models::CustomContentBodyBulk**](CustomContentBodyBulk.md)> |  | [optional]
**_links** | Option<[**models::CustomContentLinks**](CustomContentLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
