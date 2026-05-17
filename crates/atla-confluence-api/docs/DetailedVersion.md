# DetailedVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**number** | Option<**i32**> | The current version number. | [optional]
**author_id** | Option<**String**> | The account ID of the user who created this version. | [optional]
**message** | Option<**String**> | Message associated with the current version. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the version was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**minor_edit** | Option<**bool**> | Describes if this version is a minor version. Email notifications and activity stream updates are not created for minor versions. | [optional]
**content_type_modified** | Option<**bool**> | Describes if the content type is modified in this version (e.g. page to blog) | [optional]
**collaborators** | Option<**Vec<String>**> | The account IDs of users that collaborated on this version. | [optional]
**prev_version** | Option<**i32**> | The version number of the version prior to this current content update. | [optional]
**next_version** | Option<**i32**> | The version number of the version after this current content update. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
