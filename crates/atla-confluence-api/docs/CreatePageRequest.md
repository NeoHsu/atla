# CreatePageRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**space_id** | **String** | ID of the space. |
**status** | Option<**Status**> | The status of the page, published or draft. (enum: current, draft) | [optional]
**title** | Option<**String**> | Title of the page, required if page status is not draft. | [optional]
**parent_id** | Option<**String**> | The parent content ID of the page. If the `root-level` query parameter is set to false and a value is  not supplied for this parameter, then the space homepage's ID will be used. If the `root-level` query  parameter is set to true, then a value may not be supplied for this parameter. | [optional]
**body** | Option<[**models::CreatePageRequestBody**](CreatePageRequestBody.md)> |  | [optional]
**subtype** | Option<**Subtype**> | The subtype of the page. Provide the subtype live to create a live doc or no subtype to create a page. (enum: live) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
