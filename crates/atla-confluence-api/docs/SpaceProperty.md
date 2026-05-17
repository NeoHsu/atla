# SpaceProperty

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the space property. | [optional]
**key** | Option<**String**> | Key of the space property. | [optional]
**value** | Option<**serde_json::Value**> | Value of the space property. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | RFC3339 compliant date time at which the property was created. | [optional]
**created_by** | Option<**String**> | Atlassian account ID of the user that created the space property. | [optional]
**version** | Option<[**models::SpacePropertyVersion**](SpacePropertyVersion.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
