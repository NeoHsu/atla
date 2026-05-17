# CreateSpace201Response

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the space. | [optional]
**key** | Option<**String**> | Key of the space. | [optional]
**name** | Option<**String**> | Name of the space. | [optional]
**r#type** | Option<[**models::SpaceType**](SpaceType.md)> |  | [optional]
**status** | Option<[**models::SpaceStatus**](SpaceStatus.md)> |  | [optional]
**author_id** | Option<**String**> | The account ID of the user who created this space originally. | [optional]
**current_active_alias** | Option<**String**> | Currently active alias for a Confluence space. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the space was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**homepage_id** | Option<**String**> | ID of the space's homepage. | [optional]
**description** | Option<[**models::SpaceDescription**](SpaceDescription.md)> |  | [optional]
**icon** | Option<[**models::SpaceIcon**](SpaceIcon.md)> |  | [optional]
**_links** | Option<[**models::GetAttachmentById200ResponseAllOfLinks**](GetAttachmentById200ResponseAllOfLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
