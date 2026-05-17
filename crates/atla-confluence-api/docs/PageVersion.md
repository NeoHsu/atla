# PageVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the version was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**message** | Option<**String**> | Message associated with the current version. | [optional]
**number** | Option<**i32**> | The version number. | [optional]
**minor_edit** | Option<**bool**> | Describes if this version is a minor version. Email notifications and activity stream updates are not created for minor versions. | [optional]
**author_id** | Option<**String**> | The account ID of the user who created this version. | [optional]
**page** | Option<[**models::VersionedEntity**](VersionedEntity.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
