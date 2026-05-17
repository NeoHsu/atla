# \AncestorsApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_database_ancestors**](AncestorsApi.md#get_database_ancestors) | **GET** /databases/{id}/ancestors | Get all ancestors of database
[**get_folder_ancestors**](AncestorsApi.md#get_folder_ancestors) | **GET** /folders/{id}/ancestors | Get all ancestors of folder
[**get_page_ancestors**](AncestorsApi.md#get_page_ancestors) | **GET** /pages/{id}/ancestors | Get all ancestors of page
[**get_smart_link_ancestors**](AncestorsApi.md#get_smart_link_ancestors) | **GET** /embeds/{id}/ancestors | Get all ancestors of Smart Link in content tree
[**get_whiteboard_ancestors**](AncestorsApi.md#get_whiteboard_ancestors) | **GET** /whiteboards/{id}/ancestors | Get all ancestors of whiteboard



## get_database_ancestors

> models::MultiEntityResultAncestor get_database_ancestors(id, limit)
Get all ancestors of database

Returns all ancestors for a given database by ID in top-to-bottom order (that is, the highest ancestor is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results (if available) will be available by calling this endpoint with the ID of first ancestor in the response payload.  This endpoint returns minimal information about each ancestor. To fetch more details, use a related endpoint, such as [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the database and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the highest ancestor's ID to fetch the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultAncestor**](MultiEntityResult_Ancestor_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_folder_ancestors

> models::MultiEntityResultAncestor get_folder_ancestors(id, limit)
Get all ancestors of folder

Returns all ancestors for a given folder by ID in top-to-bottom order (that is, the highest ancestor is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results  (if available) will be available by calling this endpoint with the ID of first ancestor in the response payload.  This endpoint returns minimal information about each ancestor. To fetch more details, use a related endpoint, such as [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-folders-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the folder and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the folder. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the highest ancestor's ID to fetch the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultAncestor**](MultiEntityResult_Ancestor_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_ancestors

> models::MultiEntityResultAncestor1 get_page_ancestors(id, limit)
Get all ancestors of page

Returns all ancestors for a given page by ID in top-to-bottom order (that is, the highest ancestor is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results (if available) will be available by calling this endpoint with the ID of first ancestor in the response payload.  This endpoint returns minimal information about each ancestor. To fetch more details, use a related endpoint, such as [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page. | [required] |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, call this endpoint with the highest ancestor's ID to fetch the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultAncestor1**](MultiEntityResult_Ancestor__1.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_smart_link_ancestors

> models::MultiEntityResultAncestor get_smart_link_ancestors(id, limit)
Get all ancestors of Smart Link in content tree

Returns all ancestors for a given Smart Link in the content tree by ID in top-to-bottom order (that is, the highest ancestor is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results  (if available) will be available by calling this endpoint with the ID of first ancestor in the response payload.  This endpoint returns minimal information about each ancestor. To fetch more details, use a related endpoint, such as [Get Smart Link in the content tree by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the Smart Link in the content tree and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the Smart Link in the content tree. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the highest ancestor's ID to fetch the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultAncestor**](MultiEntityResult_Ancestor_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_whiteboard_ancestors

> models::MultiEntityResultAncestor get_whiteboard_ancestors(id, limit)
Get all ancestors of whiteboard

Returns all ancestors for a given whiteboard by ID in top-to-bottom order (that is, the highest ancestor is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results (if available) will be available by calling this endpoint with the ID of first ancestor in the response payload.  This endpoint returns minimal information about each ancestor. To fetch more details, use a related endpoint, such as [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the whiteboard and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the highest ancestor's ID to fetch the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultAncestor**](MultiEntityResult_Ancestor_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
