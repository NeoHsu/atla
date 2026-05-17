# \SmartLinkApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_smart_link**](SmartLinkApi.md#create_smart_link) | **POST** /embeds | Create Smart Link in the content tree
[**delete_smart_link**](SmartLinkApi.md#delete_smart_link) | **DELETE** /embeds/{id} | Delete Smart Link in the content tree
[**get_smart_link_by_id**](SmartLinkApi.md#get_smart_link_by_id) | **GET** /embeds/{id} | Get Smart Link in the content tree by id



## create_smart_link

> models::CreateSmartLink200Response create_smart_link(create_smart_link_request)
Create Smart Link in the content tree

Creates a Smart Link in the content tree in the space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the corresponding space. Permission to create a Smart Link in the content tree in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_smart_link_request** | [**CreateSmartLinkRequest**](CreateSmartLinkRequest.md) |  | [required] |

### Return type

[**models::CreateSmartLink200Response**](createSmartLink_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_smart_link

> delete_smart_link(id)
Delete Smart Link in the content tree

Delete a Smart Link in the content tree by id.  Deleting a Smart Link in the content tree moves the Smart Link to the trash, where it can be restored later  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the Smart Link in the content tree and its corresponding space. Permission to delete Smart Links in the content tree in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the Smart Link in the content tree to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_smart_link_by_id

> models::CreateSmartLink200Response get_smart_link_by_id(id, include_collaborators, include_direct_children, include_operations, include_properties)
Get Smart Link in the content tree by id

Returns a specific Smart Link in the content tree.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the Smart Link in the content tree and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the Smart Link in the content tree to be returned. | [required] |
**include_collaborators** | Option<**bool**> | Includes collaborators on the Smart Link. |  |[default to false]
**include_direct_children** | Option<**bool**> | Includes direct children of the Smart Link, as defined in the `ChildrenResponse` object. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this Smart Link in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes content properties associated with this Smart Link in the response. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]

### Return type

[**models::CreateSmartLink200Response**](createSmartLink_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
