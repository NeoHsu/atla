# \ContentApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**convert_content_ids_to_content_types**](ContentApi.md#convert_content_ids_to_content_types) | **POST** /content/convert-ids-to-types | Convert content ids to content types



## convert_content_ids_to_content_types

> models::ContentIdToContentTypeResponse convert_content_ids_to_content_types(convert_content_ids_to_content_types_request)
Convert content ids to content types

Converts a list of content ids into their associated content types. This is useful for users migrating from v1 to v2 who may have stored just content ids without their associated type. This will return types as they should be used in v2. Notably, this will return `inline-comment` for inline comments and `footer-comment` for footer comments, which is distinct from them both being represented by `comment` in v1.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the requested content. Any content that the user does not have permission to view or does not exist will map to `null` in the response.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**convert_content_ids_to_content_types_request** | [**ConvertContentIdsToContentTypesRequest**](ConvertContentIdsToContentTypesRequest.md) |  | [required] |

### Return type

[**models::ContentIdToContentTypeResponse**](ContentIdToContentTypeResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
