# PageSingle

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the page. | [optional]
**status** | Option<[**models::ContentStatus**](ContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the page. | [optional]
**space_id** | Option<**String**> | ID of the space the page is in. | [optional]
**parent_id** | Option<**String**> | ID of the parent page, or null if there is no parent page. | [optional]
**parent_type** | Option<[**models::ParentContentType**](ParentContentType.md)> |  | [optional]
**position** | Option<**i32**> | Position of child page within the given parent page tree. | [optional]
**author_id** | Option<**String**> | The account ID of the user who created this page originally. | [optional]
**owner_id** | Option<**String**> | The account ID of the user who owns this page. | [optional]
**last_owner_id** | Option<**String**> | The account ID of the user who owned this page previously, or null if there is no previous owner. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the page was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**version** | Option<[**models::Version**](Version.md)> |  | [optional]
**body** | Option<[**models::BodySingle**](BodySingle.md)> |  | [optional]
**labels** | Option<[**models::AttachmentSingleLabels**](AttachmentSingleLabels.md)> |  | [optional]
**properties** | Option<[**models::AttachmentSingleProperties**](AttachmentSingleProperties.md)> |  | [optional]
**operations** | Option<[**models::AttachmentSingleOperations**](AttachmentSingleOperations.md)> |  | [optional]
**likes** | Option<[**models::BlogPostSingleLikes**](BlogPostSingleLikes.md)> |  | [optional]
**versions** | Option<[**models::AttachmentSingleVersions**](AttachmentSingleVersions.md)> |  | [optional]
**is_favorited_by_current_user** | Option<**bool**> | Whether the page has been favorited by the current user. | [optional]
**_links** | Option<[**models::AbstractPageLinks**](AbstractPageLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
