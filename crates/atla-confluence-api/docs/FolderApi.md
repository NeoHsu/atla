# \FolderApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_folder**](FolderApi.md#create_folder) | **POST** /folders | Create folder
[**delete_folder**](FolderApi.md#delete_folder) | **DELETE** /folders/{id} | Delete folder
[**get_folder_by_id**](FolderApi.md#get_folder_by_id) | **GET** /folders/{id} | Get folder by id



## create_folder

> models::CreateFolder200Response create_folder(create_folder_request)
Create folder

Creates a folder in the space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the corresponding space. Permission to create a folder in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_folder_request** | [**CreateFolderRequest**](CreateFolderRequest.md) |  | [required] |

### Return type

[**models::CreateFolder200Response**](createFolder_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_folder

> delete_folder(id)
Delete folder

Delete a folder by id.  Deleting a folder moves the folder to the trash, where it can be restored later  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the folder and its corresponding space. Permission to delete folders in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the folder to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_folder_by_id

> models::CreateFolder200Response get_folder_by_id(id, include_collaborators, include_direct_children, include_operations, include_properties)
Get folder by id

Returns a specific folder.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the folder and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the folder to be returned. | [required] |
**include_collaborators** | Option<**bool**> | Includes collaborators on the folder. |  |[default to false]
**include_direct_children** | Option<**bool**> | Includes direct children of the folder, as defined in the `ChildrenResponse` object. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this folder in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes content properties associated with this folder in the response. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]

### Return type

[**models::CreateFolder200Response**](createFolder_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
