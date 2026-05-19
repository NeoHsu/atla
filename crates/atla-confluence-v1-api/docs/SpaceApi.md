# \SpaceApi

All URIs are relative to *http://your-domain.atlassian.net*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_space**](SpaceApi.md#delete_space) | **DELETE** /wiki/rest/api/space/{spaceKey} | Delete space
[**update_space**](SpaceApi.md#update_space) | **PUT** /wiki/rest/api/space/{spaceKey} | Update space



## delete_space

> models::LongTask delete_space(space_key)
Delete space

Permanently deletes a space without sending it to the trash. Note, the space will be deleted in a long running task. Therefore, the space may not be deleted yet when this method has returned. Clients should poll the status link that is returned in the response until the task completes.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Admin' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**space_key** | **String** | The key of the space to delete. | [required] |

### Return type

[**models::LongTask**](LongTask.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_space

> models::Space update_space(space_key, body)
Update space

Updates the name, description, or homepage of a space.  -   For security reasons, permissions cannot be updated via the API and must be changed via the user interface instead. -   Currently you cannot set space labels when updating a space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Admin' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**space_key** | **String** | The key of the space to update. | [required] |
**body** | [**SpaceUpdate**](SpaceUpdate.md) | The updated space. | [required] |

### Return type

[**models::Space**](Space.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
