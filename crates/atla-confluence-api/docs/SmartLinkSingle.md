# SmartLinkSingle

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the Smart Link in the content tree. | [optional]
**r#type** | Option<**String**> | The content type of the object. | [optional]
**status** | Option<[**models::ContentStatus**](ContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the Smart Link in the content tree. | [optional]
**parent_id** | Option<**String**> | ID of the parent content, or null if there is no parent content. | [optional]
**parent_type** | Option<[**models::ParentContentType**](ParentContentType.md)> |  | [optional]
**position** | Option<**i32**> | Position of the Smart Link within the given parent page tree. | [optional]
**author_id** | Option<**String**> | The account ID of the user who created this Smart Link in the content tree originally. | [optional]
**owner_id** | Option<**String**> | The account ID of the user who owns this Smart Link in the content tree. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the Smart Link in the content tree was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**embed_url** | Option<**String**> | The embedded URL of the Smart Link. If the Smart Link does not have an embedded URL, this property will not be included in the response. | [optional]
**space_id** | Option<**String**> | ID of the space the Smart Link is in. | [optional]
**version** | Option<[**models::Version**](Version.md)> |  | [optional]
**_links** | Option<[**models::SmartLinkLinks**](SmartLinkLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
