# \DataPoliciesApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_data_policy_metadata**](DataPoliciesApi.md#get_data_policy_metadata) | **GET** /data-policies/metadata | Get data policy metadata for the workspace
[**get_data_policy_spaces**](DataPoliciesApi.md#get_data_policy_spaces) | **GET** /data-policies/spaces | Get spaces with data policies



## get_data_policy_metadata

> models::DataPolicyMetadata get_data_policy_metadata()
Get data policy metadata for the workspace

Returns data policy metadata for the workspace.  **[Permissions](#permissions) required:** Only apps can make this request. Permission to access the Confluence site ('Can use' global permission).

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::DataPolicyMetadata**](DataPolicyMetadata.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_data_policy_spaces

> models::MultiEntityResultDataPolicySpace get_data_policy_spaces(ids, keys, sort, cursor, limit)
Get spaces with data policies

Returns all spaces. The results will be sorted by id ascending. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Only apps can make this request. Permission to access the Confluence site ('Can use' global permission). Only spaces that the app has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | Option<[**Vec<i64>**](I64.md)> | Filter the results to spaces based on their IDs. Multiple IDs can be specified as a comma-separated list. |  |
**keys** | Option<[**Vec<String>**](String.md)> | Filter the results to spaces based on their keys. Multiple keys can be specified as a comma-separated list. |  |
**sort** | Option<[**SpaceSortOrder**](SpaceSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of spaces per result to return. If more results exist, use the `Link` response header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultDataPolicySpace**](MultiEntityResult_DataPolicySpace_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
