# \WhiteboardApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_whiteboard**](WhiteboardApi.md#create_whiteboard) | **POST** /whiteboards | Create whiteboard
[**delete_whiteboard**](WhiteboardApi.md#delete_whiteboard) | **DELETE** /whiteboards/{id} | Delete whiteboard
[**get_whiteboard_by_id**](WhiteboardApi.md#get_whiteboard_by_id) | **GET** /whiteboards/{id} | Get whiteboard by id



## create_whiteboard

> models::CreateWhiteboard200Response create_whiteboard(create_whiteboard_request, private)
Create whiteboard

Creates a whiteboard in the space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the corresponding space. Permission to create a whiteboard in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_whiteboard_request** | [**CreateWhiteboardRequest**](CreateWhiteboardRequest.md) |  | [required] |
**private** | Option<**bool**> | The whiteboard will be private. Only the user who creates this whiteboard will have permission to view and edit one. |  |[default to false]

### Return type

[**models::CreateWhiteboard200Response**](createWhiteboard_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_whiteboard

> delete_whiteboard(id)
Delete whiteboard

Delete a whiteboard by id.  Deleting a whiteboard moves the whiteboard to the trash, where it can be restored later  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the whiteboard and its corresponding space. Permission to delete whiteboards in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_whiteboard_by_id

> models::CreateWhiteboard200Response get_whiteboard_by_id(id, include_collaborators, include_direct_children, include_operations, include_properties)
Get whiteboard by id

Returns a specific whiteboard.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the whiteboard and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard to be returned | [required] |
**include_collaborators** | Option<**bool**> | Includes collaborators on the whiteboard. |  |[default to false]
**include_direct_children** | Option<**bool**> | Includes direct children of the whiteboard, as defined in the `ChildrenResponse` object. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this whiteboard in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes content properties associated with this whiteboard in the response. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]

### Return type

[**models::CreateWhiteboard200Response**](createWhiteboard_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
