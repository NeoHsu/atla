# \DescendantsApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_database_descendants**](DescendantsApi.md#get_database_descendants) | **GET** /databases/{id}/descendants | Get descendants of a database
[**get_folder_descendants**](DescendantsApi.md#get_folder_descendants) | **GET** /folders/{id}/descendants | Get descendants of folder
[**get_page_descendants**](DescendantsApi.md#get_page_descendants) | **GET** /pages/{id}/descendants | Get descendants of page
[**get_smart_link_descendants**](DescendantsApi.md#get_smart_link_descendants) | **GET** /embeds/{id}/descendants | Get descendants of a smart link
[**get_whiteboard_descendants**](DescendantsApi.md#get_whiteboard_descendants) | **GET** /whiteboards/{id}/descendants | Get descendants of a whiteboard



## get_database_descendants

> models::MultiEntityResultDescendantsResponse get_database_descendants(id, limit, depth, cursor)
Get descendants of a database

Returns descendants in the content tree for a given database by ID in top-to-bottom order (that is, the highest descendant is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results (if available) will be available by calling this endpoint with the cursor in the response payload. There is also a `depth` parameter specifying depth of descendants to be fetched.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each descendant. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the database and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the cursor to fetch the next set of results. |  |[default to 25]
**depth** | Option<**i32**> | Maximum depth of descendants to return. If more results are required, use the endpoint corresponding to the content type of the deepest descendant to fetch more descendants. |  |[default to 2]
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |

### Return type

[**models::MultiEntityResultDescendantsResponse**](MultiEntityResult_DescendantsResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_folder_descendants

> models::MultiEntityResultDescendantsResponse get_folder_descendants(id, limit, depth, cursor)
Get descendants of folder

Returns descendants in the content tree for a given folder by ID in top-to-bottom order (that is, the highest descendant is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results (if available) will be available by calling this endpoint with the cursor in the response payload. There is also a `depth` parameter specifying depth of descendants to be fetched.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each descendant. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the  and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the folder. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the cursor to fetch the next set of results. |  |[default to 25]
**depth** | Option<**i32**> | Maximum depth of descendants to return. If more results are required, use the endpoint corresponding to the content type of the deepest descendant to fetch more descendants. |  |[default to 2]
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |

### Return type

[**models::MultiEntityResultDescendantsResponse**](MultiEntityResult_DescendantsResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_descendants

> models::MultiEntityResultDescendantsResponse get_page_descendants(id, limit, depth, cursor)
Get descendants of page

Returns descendants in the content tree for a given page by ID in top-to-bottom order (that is, the highest descendant is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results (if available) will be available by calling this endpoint with the cursor in the response payload. There is also a `depth` parameter specifying depth of descendants to be fetched.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each descendant. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the page and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the cursor to fetch the next set of results. |  |[default to 25]
**depth** | Option<**i32**> | Maximum depth of descendants to return. If more results are required, use the endpoint corresponding to the content type of the deepest descendant to fetch more descendants. |  |[default to 2]
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |

### Return type

[**models::MultiEntityResultDescendantsResponse**](MultiEntityResult_DescendantsResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_smart_link_descendants

> models::MultiEntityResultDescendantsResponse get_smart_link_descendants(id, limit, depth, cursor)
Get descendants of a smart link

Returns descendants in the content tree for a given smart link by ID in top-to-bottom order (that is, the highest descendant is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results (if available) will be available by calling this endpoint with the cursor in the response payload. There is also a `depth` parameter specifying depth of descendants to be fetched.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard   This endpoint returns minimal information about each descendant. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the smart link and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the smart link. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the cursor to fetch the next set of results. |  |[default to 25]
**depth** | Option<**i32**> | Maximum depth of descendants to return. If more results are required, use the endpoint corresponding to the content type of the deepest descendant to fetch more descendants. |  |[default to 2]
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |

### Return type

[**models::MultiEntityResultDescendantsResponse**](MultiEntityResult_DescendantsResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_whiteboard_descendants

> models::MultiEntityResultDescendantsResponse get_whiteboard_descendants(id, limit, depth, cursor)
Get descendants of a whiteboard

Returns descendants in the content tree for a given whiteboard by ID in top-to-bottom order (that is, the highest descendant is the first item in the response payload). The number of results is limited by the `limit` parameter and additional results (if available) will be available by calling this endpoint with the cursor in the response payload. There is also a `depth` parameter specifying depth of descendants to be fetched.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each descendant. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Permission to view the whiteboard and its corresponding space

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard. | [required] |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, call the endpoint with the cursor to fetch the next set of results. |  |[default to 25]
**depth** | Option<**i32**> | Maximum depth of descendants to return. If more results are required, use the endpoint corresponding to the content type of the deepest descendant to fetch more descendants. |  |[default to 2]
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |

### Return type

[**models::MultiEntityResultDescendantsResponse**](MultiEntityResult_DescendantsResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
