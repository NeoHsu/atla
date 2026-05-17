# FolderSingle

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the folder. | [optional]
**r#type** | Option<**String**> | The content type of the object. | [optional]
**status** | Option<[**models::ContentStatus**](ContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the folder. | [optional]
**parent_id** | Option<**String**> | ID of the parent content, or null if there is no parent content. | [optional]
**parent_type** | Option<[**models::ParentContentType**](ParentContentType.md)> |  | [optional]
**position** | Option<**i32**> | Position of the folder within the given parent page tree. | [optional]
**author_id** | Option<**String**> | The account ID of the user who created this folder. | [optional]
**owner_id** | Option<**String**> | The account ID of the user who owns this folder. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the folder was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**space_id** | Option<**String**> | ID of the space the folder is in. | [optional]
**version** | Option<[**models::Version**](Version.md)> |  | [optional]
**_links** | Option<[**models::FolderLinks**](FolderLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
