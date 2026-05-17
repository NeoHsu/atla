# SpaceIcon

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**path** | Option<**String**> | The path (relative to base URL) at which the space's icon can be retrieved. The format should be like `/wiki/download/...` or `/wiki/aa-avatar/...` | [optional]
**api_download_link** | Option<**String**> | The path (relative to base URL) that can be used to retrieve a link to download the space icon. 3LO apps should use this link instead of the value provided in the `path` property to retrieve the icon.  Currently this field is only returned for `global` spaces and not `personal` spaces.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
