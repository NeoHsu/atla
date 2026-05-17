# ContentIdToContentTypeResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**results** | Option<[**std::collections::HashMap<String, models::ContentIdToContentTypeResponseResultsValue>**](ContentIdToContentTypeResponseResultsValue.md)> | JSON object containing all requested content ids as keys and their associated content types as the values. Duplicate content ids in the request will be returned under a single key in the response. For built-in content types, the enumerations are as specified. Custom content ids will be mapped to their associated type. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
